//! Tests of the `reqwest-middleware` integration.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};

use http::{Extensions, HeaderValue};
use pretix::{Client, Error, RetryPolicy};
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

type BoxFuture<'a> =
    Pin<Box<dyn Future<Output = reqwest_middleware::Result<Response>> + Send + 'a>>;

fn client(server: &MockServer, retry: RetryPolicy, middleware: impl Middleware) -> Client {
    let http = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
        .with(middleware)
        .build();
    Client::builder()
        .base_url(format!("{}/api/v1/", server.uri()))
        .token("secret")
        .retry_policy(retry)
        .middleware_client(http)
        .build()
        .unwrap()
}

fn tag<'a>(mut req: Request, extensions: &'a mut Extensions, next: Next<'a>) -> BoxFuture<'a> {
    req.headers_mut()
        .insert("x-middleware", HeaderValue::from_static("yes"));
    Box::pin(next.run(req, extensions))
}

#[tokio::test]
async fn sends_requests_through_middleware() {
    let server = MockServer::start().await;
    let client = client(&server, RetryPolicy::disabled(), tag);
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/acme/"))
        .and(header("x-middleware", "yes"))
        .and(header("authorization", "Token secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "name": "acme", "slug": "acme", "public_url": null, "plugins": [],
        })))
        .expect(1)
        .mount(&server)
        .await;

    client.organizers().get("acme").await.unwrap();
}

static REJECTED: AtomicUsize = AtomicUsize::new(0);

fn reject<'a>(_: Request, _: &'a mut Extensions, _: Next<'a>) -> BoxFuture<'a> {
    REJECTED.fetch_add(1, Ordering::SeqCst);
    Box::pin(async {
        Err(reqwest_middleware::Error::middleware(
            std::io::Error::other("rejected"),
        ))
    })
}

#[tokio::test]
async fn middleware_errors_are_not_retried() {
    let server = MockServer::start().await;
    let client = client(&server, RetryPolicy::default().with_max_retries(3), reject);

    let err = client.organizers().get("acme").await.unwrap_err();
    let Error::Transport(transport) = &err else {
        panic!("expected a transport error, got {err:?}");
    };
    assert!(transport.is_middleware());
    assert!(!err.is_transient() && !err.is_retryable());
    assert_eq!(REJECTED.load(Ordering::SeqCst), 1);
}
