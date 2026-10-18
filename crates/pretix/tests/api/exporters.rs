use std::time::Duration;

use pretix::resources::exporters::{ExportJob, Exporter};
use pretix::{JobStatus, ListParams, PollOptions};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn exporter_models() {
    let exporter = assert_model::<Exporter>(&fixture("exporters/exporter"), &[]);
    assert_eq!(
        exporter.input_parameters[0].choices.as_ref().unwrap()[0],
        "xlsx"
    );
    assert_eq!(exporter.input_parameters[1].choices, None);
    assert_model::<Exporter>(&fixture("exporters/organizer_exporter"), &[]);
    assert_model::<ExportJob>(&fixture("exporters/export_job"), &[]);
}

#[tokio::test]
async fn list_event_and_organizer_exporters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/exporters/",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("exporters/exporter")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/exporters/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("exporters/organizer_exporter")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let org = client.organizer("bigevents");
    let event = org
        .event("sampleconf")
        .exporters()
        .list(&ListParams::default())
        .await
        .unwrap();
    assert_eq!(event.results[0].identifier, "orderlist");
    let organizer = org.exporters().list(&ListParams::default()).await.unwrap();
    assert_eq!(organizer.results[0].input_parameters[0].name, "events");
}

#[tokio::test]
async fn run_and_download() {
    let (server, client) = setup().await;
    let download_path = "/api/v1/organizers/bigevents/exporters/orderlist/download/a/b/";
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/exporters/orderlist/run/",
        ))
        .and(body_json(json!({"_format": "xlsx"})))
        .respond_with(
            ResponseTemplate::new(202)
                .set_body_json(json!({"download": format!("https://pretix.eu{download_path}")})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/organizers/bigevents/exporters/orderlist/run/",
        ))
        .and(body_json(
            json!({"_format": "xlsx", "events": ["sampleconf"]}),
        ))
        .respond_with(
            ResponseTemplate::new(202)
                .set_body_json(json!({"download": format!("https://pretix.eu{download_path}")})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(download_path))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({"status": "waiting"})))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(download_path))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header(
                    "Content-Disposition",
                    "attachment; filename=\"orders.xlsx\"",
                )
                .set_body_bytes("xlsx"),
        )
        .mount(&server)
        .await;

    let org = client.organizer("bigevents");
    let job = org
        .event("sampleconf")
        .exporters()
        .run("orderlist", &json!({"_format": "xlsx"}))
        .await
        .unwrap();
    // The download link is followed on the configured origin.
    let status = job.download_job(&client).await.unwrap();
    assert_eq!(status, JobStatus::Pending { percentage: None });
    let download = job
        .download_job(&client)
        .wait(PollOptions::default().with_interval(Duration::from_millis(1)))
        .await
        .unwrap();
    assert_eq!(download.filename.as_deref(), Some("orders.xlsx"));

    let job = org
        .exporters()
        .run(
            "orderlist",
            &json!({"_format": "xlsx", "events": ["sampleconf"]}),
        )
        .await
        .unwrap();
    assert_eq!(job.download.path(), download_path);
}
