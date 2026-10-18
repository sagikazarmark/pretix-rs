//! Tests of the shared request machinery, using organizers/events as carriers.

use std::future::IntoFuture;
use std::time::Duration;

use futures_util::TryStreamExt;
use http::StatusCode;
use pretix::resources::events::ListEvents;
use pretix::resources::organizers::{ListOrganizers, Organizer, UpdateOrganizer};
use pretix::{Client, Conditional, Error, JobStatus, PageCursor, PollOptions, RetryPolicy};
use serde_json::json;
use wiremock::matchers::{body_bytes, body_json, header, header_exists, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{fixture, setup};

fn organizer(slug: &str) -> serde_json::Value {
    json!({"name": slug, "slug": slug, "public_url": null, "plugins": []})
}

#[tokio::test]
async fn sends_standard_headers() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/acme/"))
        .and(header("authorization", "Token secret"))
        .and(header("accept", "application/json"))
        .and(header_exists("user-agent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(organizer("acme")))
        .expect(1)
        .mount(&server)
        .await;
    client.organizers().get("acme").await.unwrap();
}

#[tokio::test]
async fn escapes_path_segments() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/a%2Fb/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(organizer("a/b")))
        .expect(1)
        .mount(&server)
        .await;
    client.organizers().get("a/b").await.unwrap();
}

#[tokio::test]
async fn follows_pagination_on_configured_origin() {
    let (server, client) = setup().await;
    // `next` points at a different origin (e.g. behind a reverse proxy): the client
    // must follow it on the configured origin instead of leaking the token.
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "count": 3, "next": null, "previous": "http://evil.example/api/v1/organizers/",
            "results": [organizer("c")],
        })))
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/"))
        .and(query_param("page_size", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "count": 3, "next": "http://evil.example/api/v1/organizers/?page=2&page_size=2", "previous": null,
            "results": [organizer("a"), organizer("b")],
        })))
        .expect(2)
        .mount(&server)
        .await;

    let params = ListOrganizers::builder().page_size(2).build();
    let all = client.organizers().list(&params).all().await.unwrap();
    let slugs: Vec<_> = all.iter().map(|o| o.slug.as_str()).collect();
    assert_eq!(slugs, ["a", "b", "c"]);

    let streamed: Vec<Organizer> = client
        .organizers()
        .list(&params)
        .stream()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(streamed.len(), 3);
}

#[tokio::test]
async fn manual_pagination_with_serializable_cursor() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "count": 1, "next": null, "previous": null, "results": [organizer("z")],
        })))
        .expect(1)
        .mount(&server)
        .await;

    let cursor: PageCursor =
        serde_json::from_value(json!("https://pretix.eu/api/v1/organizers/?page=2")).unwrap();
    let page = client.page::<Organizer>(&cursor).await.unwrap();
    assert_eq!(page.results[0].slug, "z");
    assert!(page.is_last());
}

#[tokio::test]
async fn exposes_response_meta() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/acme/events/"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("X-Page-Generated", "2017-12-01T10:00:00Z")
                .insert_header("Last-Modified", "Wed, 21 Oct 2015 07:28:00 GMT")
                .set_body_json(json!({"count": 0, "next": null, "previous": null, "results": []})),
        )
        .mount(&server)
        .await;
    let page = client
        .organizer("acme")
        .events()
        .list(&ListEvents::default())
        .await
        .unwrap();
    assert_eq!(
        page.meta.page_generated,
        Some("2017-12-01T10:00:00Z".parse().unwrap())
    );
    assert_eq!(
        page.meta.last_modified,
        Some("2015-10-21T07:28:00Z".parse().unwrap())
    );
}

#[tokio::test]
async fn conditional_requests() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/acme/events/"))
        // `header()` splits values on commas, which HTTP dates contain.
        .and(|request: &wiremock::Request| {
            request
                .headers
                .get("if-modified-since")
                .and_then(|v| v.to_str().ok())
                == Some("Wed, 21 Oct 2015 07:28:00 GMT")
        })
        .respond_with(ResponseTemplate::new(304))
        .expect(1)
        .mount(&server)
        .await;
    let result = client
        .organizer("acme")
        .events()
        .list(&ListEvents::default())
        .if_modified_since("2015-10-21T07:28:00Z".parse().unwrap())
        .await
        .unwrap();
    assert!(matches!(result, Conditional::NotModified));
}

#[tokio::test]
async fn parses_api_errors() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/forbidden/"))
        .respond_with(
            ResponseTemplate::new(403)
                .set_body_json(json!({"detail": "You do not have permission."})),
        )
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/organizers/invalid/"))
        .respond_with(
            ResponseTemplate::new(400).set_body_json(json!({"plugins": ["Unknown plugin."]})),
        )
        .mount(&server)
        .await;

    let err = client.organizers().get("forbidden").await.unwrap_err();
    assert_eq!(err.status(), Some(StatusCode::FORBIDDEN));
    assert!(!err.is_retryable());
    assert_eq!(
        err.as_api().unwrap().detail(),
        Some("You do not have permission.")
    );
    assert!(err.to_string().contains("You do not have permission."));

    let err = client
        .organizers()
        .update("invalid", &UpdateOrganizer::default())
        .await
        .unwrap_err();
    let fields = err.as_api().unwrap().field_errors().unwrap();
    assert_eq!(fields["plugins"][0], "Unknown plugin.");
}

#[tokio::test]
async fn reports_decode_error_path() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "count": 1, "next": null, "previous": null, "results": [{"name": 1, "slug": "x"}],
        })))
        .mount(&server)
        .await;
    let err = client
        .organizers()
        .list(&ListOrganizers::default())
        .await
        .unwrap_err();
    match err {
        Error::Decode { path, .. } => assert_eq!(path, "results[0].name"),
        other => panic!("unexpected error {other:?}"),
    }
}

fn retrying_client(server: &MockServer) -> Client {
    Client::builder()
        .base_url(server.uri())
        .token("secret")
        .retry_policy(RetryPolicy::default().with_initial_backoff(Duration::from_millis(1)))
        .build()
        .unwrap()
}

#[tokio::test]
async fn retries_rate_limited_requests() {
    let server = MockServer::start().await;
    let client = retrying_client(&server);
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/acme/"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
        .up_to_n_times(2)
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/acme/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(organizer("acme")))
        .expect(1)
        .mount(&server)
        .await;
    client.organizers().get("acme").await.unwrap();
}

#[tokio::test]
async fn does_not_retry_unkeyed_mutations() {
    let server = MockServer::start().await;
    let client = retrying_client(&server);
    Mock::given(method("DELETE"))
        .and(path("/api/v1/organizers/acme/events/demo/"))
        .respond_with(ResponseTemplate::new(503).insert_header("Retry-After", "0"))
        .expect(1)
        .mount(&server)
        .await;
    let err = client
        .organizer("acme")
        .events()
        .delete("demo")
        .await
        .unwrap_err();
    assert_eq!(err.status(), Some(StatusCode::SERVICE_UNAVAILABLE));
    // Transient, but not safe to repeat: the DELETE may have been executed.
    assert!(err.is_transient());
    assert!(!err.is_retryable());
    assert_eq!(err.retry_after(), Some(Duration::ZERO));
}

#[tokio::test]
async fn retries_keyed_mutations_and_sends_key() {
    let server = MockServer::start().await;
    let client = retrying_client(&server);
    Mock::given(method("DELETE"))
        .and(path("/api/v1/organizers/acme/events/demo/"))
        .and(header("x-idempotency-key", "key-1"))
        .respond_with(ResponseTemplate::new(409).insert_header("Retry-After", "0"))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/organizers/acme/events/demo/"))
        .and(header("x-idempotency-key", "key-1"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    client
        .organizer("acme")
        .events()
        .delete("demo")
        .idempotency_key("key-1")
        .await
        .unwrap();
}

#[tokio::test]
async fn polls_jobs() {
    let (server, client) = setup().await;
    let url = format!(
        "{}/api/v1/organizers/acme/exporters/orderlist/download/a/b/",
        server.uri()
    );
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/acme/exporters/orderlist/download/a/b/",
        ))
        .respond_with(
            ResponseTemplate::new(409)
                .set_body_json(json!({"status": "running", "percentage": 40})),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/acme/exporters/orderlist/download/a/b/",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "text/csv")
                .insert_header("Content-Disposition", "attachment; filename=\"orders.csv\"")
                .set_body_bytes("a,b\n"),
        )
        .mount(&server)
        .await;

    let url = url.parse().unwrap();
    let status = client.download_job(&url).await.unwrap();
    assert_eq!(
        status,
        JobStatus::Pending {
            percentage: Some(40.0)
        }
    );

    let download = client
        .download_job(&url)
        .wait(PollOptions::default().with_interval(Duration::from_millis(1)))
        .await
        .unwrap();
    assert_eq!(download.filename.as_deref(), Some("orders.csv"));
    assert_eq!(download.content_type.as_deref(), Some("text/csv"));
    assert_eq!(&download.data[..], b"a,b\n");
}

#[tokio::test]
async fn failed_jobs() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/job/"))
        .respond_with(
            ResponseTemplate::new(410)
                .set_body_json(json!({"status": "failed", "message": "Boom"})),
        )
        .mount(&server)
        .await;
    let url = format!("{}/api/v1/job/", server.uri()).parse().unwrap();
    let err = client
        .download_job(&url)
        .wait(PollOptions::default())
        .await
        .unwrap_err();
    assert!(matches!(err, Error::JobFailed(message) if message == "Boom"));
}

#[tokio::test]
async fn uploads_files() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/upload"))
        .and(header("content-type", "image/png"))
        .and(header(
            "content-disposition",
            "attachment; filename=\"logo.png\"",
        ))
        .and(body_bytes(b"PNG".to_vec()))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"id": "file:1cd99455"})))
        .expect(1)
        .mount(&server)
        .await;
    let id = client
        .upload("logo.png", "image/png", &b"PNG"[..])
        .await
        .unwrap();
    assert_eq!(id.as_str(), "file:1cd99455");
}

#[tokio::test]
async fn decode_as_escape_hatch() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/acme/events/sampleconf/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("events/event")))
        .mount(&server)
        .await;
    let raw = client
        .organizer("acme")
        .events()
        .get("sampleconf")
        .decode_as::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(raw["timezone"], "Europe/Berlin");
}

/// Futures and streams must be `Send + 'static` (e.g. for durable execution engines
/// and `tokio::spawn`), errors `Send + Sync + 'static`.
#[allow(dead_code)]
fn futures_are_send_and_static(client: &Client) {
    fn assert_send<T: Send + 'static>(_: T) {}
    fn assert_error<T: std::error::Error + Send + Sync + 'static>() {}
    assert_error::<Error>();
    let events = client.organizer("acme").events();
    assert_send(events.get("x").into_future());
    assert_send(events.list(&ListEvents::default()).all());
    assert_send(events.list(&ListEvents::default()).stream());
    assert_send(events.list(&ListEvents::default()).pages());
    assert_send(
        events
            .list(&ListEvents::default())
            .if_modified_since(jiff::Timestamp::UNIX_EPOCH)
            .into_future(),
    );
    assert_send(client.upload("a.png", "image/png", vec![1u8]).into_future());
    assert_send(
        client
            .download_job(&"https://x/".parse().unwrap())
            .wait(PollOptions::default()),
    );
}

#[tokio::test]
async fn requests_can_be_spawned() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/acme/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(organizer("acme")))
        .mount(&server)
        .await;
    let request = client.organizers().get("acme");
    drop(client);
    let organizer = tokio::spawn(request.into_future()).await.unwrap().unwrap();
    assert_eq!(organizer.slug, "acme");
}

#[tokio::test]
async fn redirects_are_not_followed() {
    let server = MockServer::start().await;
    let client = Client::builder()
        .base_url(server.uri())
        .token("secret")
        .build()
        .unwrap();
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/acme/events/demo/clone/"))
        .respond_with(ResponseTemplate::new(301).insert_header("Location", "/elsewhere/"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/elsewhere/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(0)
        .mount(&server)
        .await;
    let body = pretix::resources::events::CreateEvent::builder()
        .name(pretix::types::LocalizedString::from([("en", "Demo")]))
        .slug("demo2")
        .currency("EUR")
        .date_from(jiff::Timestamp::UNIX_EPOCH)
        .build();
    let err = client
        .organizer("acme")
        .events()
        .clone_event("demo", &body)
        .await
        .unwrap_err();
    assert_eq!(err.status(), Some(StatusCode::MOVED_PERMANENTLY));
}

#[tokio::test]
async fn detects_pagination_loops() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "count": 2, "next": format!("{}/api/v1/organizers/", server.uri()), "previous": null,
            "results": [organizer("a")],
        })))
        .expect(1)
        .mount(&server)
        .await;
    let err = client
        .organizers()
        .list(&ListOrganizers::default())
        .all()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::UnexpectedResponse(_)), "{err:?}");
}

#[tokio::test]
async fn huge_counts_do_not_preallocate() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "count": u64::MAX, "next": null, "previous": null, "results": [organizer("a")],
        })))
        .mount(&server)
        .await;
    let all = client
        .organizers()
        .list(&ListOrganizers::default())
        .all()
        .await
        .unwrap();
    assert_eq!(all.len(), 1);
}

#[tokio::test]
async fn retries_rate_limited_mutations() {
    let server = MockServer::start().await;
    let client = retrying_client(&server);
    Mock::given(method("DELETE"))
        .and(path("/api/v1/organizers/acme/events/demo/"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/organizers/acme/events/demo/"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    client
        .organizer("acme")
        .events()
        .delete("demo")
        .await
        .unwrap();
}

#[tokio::test]
async fn does_not_wait_for_long_retry_after() {
    let server = MockServer::start().await;
    let client = retrying_client(&server);
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/acme/"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "3600"))
        .expect(1)
        .mount(&server)
        .await;
    let err = client.organizers().get("acme").await.unwrap_err();
    assert_eq!(err.retry_after(), Some(Duration::from_secs(3600)));
    assert!(err.is_retryable());
}

#[tokio::test]
async fn classifies_transport_errors() {
    let client = Client::builder()
        .base_url("http://127.0.0.1:9/api/v1/")
        .token("secret")
        .retry_policy(RetryPolicy::default().with_initial_backoff(Duration::from_millis(1)))
        .build()
        .unwrap();
    let err = client.organizers().get("acme").await.unwrap_err();
    assert!(matches!(err, Error::Transport(_)));
    assert!(err.is_transient() && err.is_retryable());
    assert!(err.to_string().contains("HTTP transport error"));

    let err = client
        .organizer("acme")
        .events()
        .delete("demo")
        .await
        .unwrap_err();
    assert!(err.is_transient() && !err.is_retryable());
}

#[tokio::test]
async fn custom_requests() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/acme/my_plugin/"))
        .and(query_param("dry_run", "true"))
        .and(body_json(json!({"x": 1})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .expect(1)
        .mount(&server)
        .await;
    let value: serde_json::Value = client
        .request(http::Method::POST, "organizers/acme/my_plugin/")
        .query(&[("dry_run", "true")])
        .json_body(&json!({"x": 1}))
        .json()
        .await
        .unwrap();
    assert_eq!(value["ok"], true);
}

#[test]
fn core_types_roundtrip() {
    fn roundtrip<
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    >(
        value: &T,
    ) {
        let json = serde_json::to_string(value).unwrap();
        assert_eq!(&serde_json::from_str::<T>(&json).unwrap(), value);
    }
    let mut page = pretix::Page::new(vec![1, 2, 3]);
    page.next = Some(
        "https://pretix.eu/api/v1/organizers/?page=2"
            .parse()
            .unwrap(),
    );
    roundtrip(&page);
    roundtrip(&page.next_cursor().unwrap());
    roundtrip(&JobStatus::<u32>::Pending {
        percentage: Some(12.5),
    });
    roundtrip(&JobStatus::<u32>::Failed { message: None });
    roundtrip(&JobStatus::Ready(7u32));
    roundtrip(&Conditional::<u32>::NotModified);
    roundtrip(&Conditional::Modified(7u32));
}
