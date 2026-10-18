use pretix::resources::shipping::{
    ListPrintJobs, PollPrintJobs, PrintJob, ShippingAddress, ShippingMethodType, ShippingState,
    ShippingStatus,
};
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn shipping_models() {
    assert_model::<ShippingAddress>(&fixture("shipping/shipping_address"), &[]);
    let status = assert_model::<ShippingStatus>(&fixture("shipping/shipping_status"), &[]);
    assert_eq!(status.method_type, ShippingMethodType::Ship);
    let job = assert_model::<PrintJob>(&fixture("shipping/print_job"), &[]);
    assert_eq!(job.order, "ABC12");
    // The field table documents the order code as `code`.
    let job: PrintJob = serde_json::from_value(
        json!({"event": "democon", "code": "ABC12", "method": "ship", "status": "new"}),
    )
    .unwrap();
    assert_eq!(job.order, "ABC12");
}

#[tokio::test]
async fn shipping_address_and_status() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/democon/orders/ABC12/shippingaddress/",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("shipping/shipping_address")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/democon/orders/ABC12/shippingstatus/",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("shipping/shipping_status")))
        .expect(1)
        .mount(&server)
        .await;

    let shipping = client.organizer("bigevents").event("democon").shipping();
    let address = shipping.shipping_address("ABC12").await.unwrap();
    assert_eq!(address.city, "Berlin");
    let status = shipping.shipping_status("ABC12").await.unwrap();
    assert_eq!(status.status, ShippingState::New);
}

#[tokio::test]
async fn list_print_jobs() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/printjobs/"))
        .and(query_param("method", "ship"))
        .and(query_param("method", "collect"))
        .and(query_param("status", "new"))
        .and(query_param("event", "democon"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("shipping/print_job")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListPrintJobs::builder()
        .method(vec![ShippingMethodType::Ship, ShippingMethodType::Collect])
        .status(ShippingState::New)
        .event("democon")
        .build();
    let page = client
        .organizer("bigevents")
        .print_jobs()
        .list(&params)
        .await
        .unwrap();
    assert_eq!(page.results[0].event, "democon");
}

#[tokio::test]
async fn poll_and_ack() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/printjobs/poll/"))
        .and(query_param("method", "ship"))
        .and(query_param("status", "new"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/pdf")
                .insert_header("x-pretix-order-code", "ABC12")
                .set_body_bytes(b"%PDF".to_vec()),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/bigevents/printjobs/ABC12/ack/"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let jobs = client.organizer("bigevents").print_jobs();
    let params = PollPrintJobs::builder()
        .method(vec![ShippingMethodType::Ship])
        .status(ShippingState::New)
        .build();
    let job = jobs.poll(&params).await.unwrap();
    assert_eq!(job.order.as_deref(), Some("ABC12"));
    assert_eq!(job.file.content_type.as_deref(), Some("application/pdf"));
    assert_eq!(&job.file.data[..], b"%PDF");
    jobs.ack("ABC12").await.unwrap();
}
