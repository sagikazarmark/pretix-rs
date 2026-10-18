use pretix::resources::exporters::ExportJob;
use pretix::resources::offline_sales::{
    CreateOfflineSalesBatch, OfflineSalesBatch, OfflineSalesBatchCreation,
};
use pretix::{JobStatus, ListParams, PollOptions};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const BATCHES: &str = "/api/v1/organizers/bigevents/events/sampleconf/offlinesalesbatches/";

#[test]
fn offline_sales_models() {
    let batch = assert_model::<OfflineSalesBatch>(&fixture("offline_sales/batch_in_list"), &[]);
    assert_eq!(batch.orders, Vec::<String>::new());
    let batch = assert_model::<OfflineSalesBatch>(&fixture("offline_sales/batch"), &[]);
    assert_eq!(batch.orders.len(), 7);
    assert_model::<OfflineSalesBatchCreation>(&fixture("offline_sales/creation"), &[]);
    assert_model::<ExportJob>(&fixture("offline_sales/rendering"), &[]);
}

#[tokio::test]
async fn list_and_get() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(BATCHES))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("offline_sales/batch_in_list")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{BATCHES}1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("offline_sales/batch")))
        .expect(1)
        .mount(&server)
        .await;

    let batches = client
        .organizer("bigevents")
        .event("sampleconf")
        .offline_sales_batches();
    let page = batches.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].amount, 7);
    let batch = batches.get(1).await.unwrap();
    assert_eq!(batch.orders[0], "TSRNN");
}

#[tokio::test]
async fn create_and_check() {
    let (server, client) = setup().await;
    let check = format!(
        "{}{BATCHES}check/29891ede-196f-4942-9e26-d055a36e98b8/",
        server.uri()
    );
    Mock::given(method("POST"))
        .and(path(BATCHES))
        .and(body_json(json!({
            "item": 14,
            "layout": 123,
            "amount": 10,
            "testmode": true,
            "sales_channel": "web",
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({"check": check})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!(
            "{BATCHES}check/29891ede-196f-4942-9e26-d055a36e98b8/"
        )))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({"status": "running"})))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!(
            "{BATCHES}check/29891ede-196f-4942-9e26-d055a36e98b8/"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("offline_sales/batch")))
        .expect(1)
        .mount(&server)
        .await;

    let batches = client
        .organizer("bigevents")
        .event("sampleconf")
        .offline_sales_batches();
    let body = CreateOfflineSalesBatch::builder()
        .item(14)
        .layout(123)
        .amount(10)
        .testmode(true)
        .sales_channel("web")
        .build();
    let created = batches.create(&body).await.unwrap();
    assert_eq!(created.check.as_str(), check);

    let status = batches.check(&created).await.unwrap();
    assert_eq!(status, JobStatus::Pending { percentage: None });
    let batch = batches
        .check(&created)
        .wait(PollOptions::default())
        .await
        .unwrap();
    assert_eq!(batch.id, 1);
}

#[tokio::test]
async fn check_failed() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(format!("{BATCHES}check/abc/")))
        .respond_with(
            ResponseTemplate::new(410)
                .set_body_json(json!({"status": "failed", "message": "Quota sold out"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let job: OfflineSalesBatchCreation =
        serde_json::from_value(json!({"check": format!("{}{BATCHES}check/abc/", server.uri())}))
            .unwrap();
    let status = client
        .organizer("bigevents")
        .event("sampleconf")
        .offline_sales_batches()
        .check(&job)
        .await
        .unwrap();
    assert_eq!(
        status,
        JobStatus::Failed {
            message: Some("Quota sold out".into())
        }
    );
}

#[tokio::test]
async fn render_and_download() {
    let (server, client) = setup().await;
    let download_path = format!("{BATCHES}1/download/29891ede/3f279f13/");
    Mock::given(method("POST"))
        .and(path(format!("{BATCHES}1/render/")))
        .respond_with(
            ResponseTemplate::new(202)
                .set_body_json(json!({"download": format!("{}{download_path}", server.uri())})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(download_path))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/pdf")
                .set_body_bytes(b"%PDF".to_vec()),
        )
        .expect(1)
        .mount(&server)
        .await;

    let rendering = client
        .organizer("bigevents")
        .event("sampleconf")
        .offline_sales_batches()
        .render(1)
        .await
        .unwrap();
    let file = rendering
        .download_job(&client)
        .await
        .unwrap()
        .ready()
        .unwrap();
    assert_eq!(&file.data[..], b"%PDF");
}
