use std::time::Duration;

use pretix::resources::shredders::{
    ExportShredders, ShredJob, ShredOutcome, Shredder, ShredderExport,
};
use pretix::{Error, JobStatus, ListParams, PollOptions};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const BASE: &str = "/api/v1/organizers/bigevents/events/sampleconf/shredders/";

#[test]
fn shredder_models() {
    assert_model::<Shredder>(&fixture("shredders/shredder"), &[]);
    assert_model::<ShredderExport>(&fixture("shredders/shredder_export"), &[]);
    assert_model::<ShredJob>(&fixture("shredders/shred_job"), &[]);
}

#[tokio::test]
async fn list_export_shred_and_check() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(BASE))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("shredders/shredder")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{BASE}export/")))
        .and(body_json(json!({"shredders": ["question_answers"]})))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "download": format!("https://pretix.eu{BASE}download/a/b/"),
            "shred": format!("https://pretix.eu{BASE}shred/a/b/"),
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{BASE}download/a/b/")))
        .respond_with(ResponseTemplate::new(200).set_body_bytes("zip"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{BASE}shred/a/b/")))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({"status": "running"})))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{BASE}shred/a/b/")))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "status": format!("https://pretix.eu{BASE}status/a/b/"),
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{BASE}status/a/b/")))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({"status": "waiting"})))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{BASE}status/a/b/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status": "ok"})))
        .mount(&server)
        .await;

    let shredders = client
        .organizer("bigevents")
        .event("sampleconf")
        .shredders();
    let page = shredders.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].identifier, "question_answers");

    let export = shredders
        .export(
            &ExportShredders::builder()
                .shredders(vec!["question_answers".into()])
                .build(),
        )
        .await
        .unwrap();
    let download = export.download_job(&client).await.unwrap().ready().unwrap();
    assert_eq!(&download.data[..], b"zip");

    let status = shredders.shred(&export).await.unwrap();
    assert!(status.is_pending());
    let job = shredders.shred(&export).await.unwrap().ready().unwrap();

    let outcome = shredders
        .status(&job)
        .wait(PollOptions::default().with_interval(Duration::from_millis(1)))
        .await
        .unwrap();
    assert_eq!(outcome, ShredOutcome::Completed);
}

#[tokio::test]
async fn status_codes() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(format!("{BASE}status/gone/x/")))
        .respond_with(ResponseTemplate::new(410))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{BASE}status/failed/x/")))
        .respond_with(
            ResponseTemplate::new(417)
                .set_body_json(json!({"status": "failed", "message": "Boom"})),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{BASE}status/missing/x/")))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let shredders = client
        .organizer("bigevents")
        .event("sampleconf")
        .shredders();
    let job = |id: &str| -> ShredJob {
        serde_json::from_value(json!({"status": format!("{}{BASE}status/{id}/x/", server.uri())}))
            .unwrap()
    };
    assert_eq!(
        shredders.status(&job("gone")).await.unwrap(),
        JobStatus::Ready(ShredOutcome::Unknown)
    );
    assert_eq!(
        shredders.status(&job("failed")).await.unwrap(),
        JobStatus::Failed {
            message: Some("Boom".into())
        }
    );
    let err = shredders
        .status(&job("failed"))
        .wait(PollOptions::default())
        .await
        .unwrap_err();
    assert!(matches!(err, Error::JobFailed(message) if message == "Boom"));
    let err = shredders.status(&job("missing")).await.unwrap_err();
    assert_eq!(err.status(), Some(http::StatusCode::NOT_FOUND));
}
