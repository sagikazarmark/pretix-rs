use std::time::Duration;

use pretix::PollOptions;
use wiremock::matchers::{method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

use crate::support::setup;

const CERTIFICATE: &str =
    "/api/v1/organizers/bigevents/events/sampleconf/orderpositions/23442/certificate/";
const RESULT: &str = "1f550651-ae7b-4911-a76c-2be8f348aaa5";

#[tokio::test]
async fn start_and_poll() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(CERTIFICATE))
        .and(query_param_is_missing("result"))
        .respond_with(
            ResponseTemplate::new(303)
                .insert_header("location", format!("{CERTIFICATE}?result={RESULT}")),
        )
        // Only the initial request may start a job.
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(CERTIFICATE))
        .and(query_param("result", RESULT))
        .respond_with(ResponseTemplate::new(409))
        .up_to_n_times(2)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(CERTIFICATE))
        .and(query_param("result", RESULT))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/pdf")
                .set_body_bytes(b"%PDF".to_vec()),
        )
        .mount(&server)
        .await;

    let job = client
        .organizer("bigevents")
        .event("sampleconf")
        .certificates()
        .start(23442)
        .await
        .unwrap();
    assert_eq!(
        job.download.query(),
        Some(format!("result={RESULT}").as_str())
    );

    let download = job
        .download_job(&client)
        .wait(PollOptions::default().with_interval(Duration::from_millis(1)))
        .await
        .unwrap();
    assert_eq!(download.content_type.as_deref(), Some("application/pdf"));
    assert_eq!(&download.data[..], b"%PDF");
}

#[tokio::test]
async fn start_without_following_redirects() {
    let server = wiremock::MockServer::start().await;
    let client = pretix::Client::builder()
        .base_url(format!("{}/api/v1/", server.uri()))
        .token("secret")
        .http_client(
            reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();
    Mock::given(method("GET"))
        .and(path(CERTIFICATE))
        .respond_with(
            ResponseTemplate::new(303)
                .insert_header("location", format!("{CERTIFICATE}?result={RESULT}")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let job = client
        .organizer("bigevents")
        .event("sampleconf")
        .certificates()
        .start(23442)
        .await
        .unwrap();
    assert_eq!(job.download.path(), CERTIFICATE);
    assert_eq!(
        job.download.query(),
        Some(format!("result={RESULT}").as_str())
    );
}
