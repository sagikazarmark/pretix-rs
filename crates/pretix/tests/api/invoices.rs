use pretix::query::Ordering;
use pretix::resources::invoices::{
    Invoice, InvoiceOrdering, ListInvoices, TransmissionStatus, TransmissionType,
};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const EVENT: &str = "/api/v1/organizers/bigevents/events/sampleconf";

#[test]
fn invoice_model() {
    let invoice = assert_model::<Invoice>(&fixture("invoices/invoice"), &[]);
    assert_eq!(invoice.transmission_type, Some(TransmissionType::Email));
    assert_eq!(
        invoice.transmission_status,
        Some(TransmissionStatus::Completed)
    );
    assert_eq!(invoice.lines[0].gross_value.to_string(), "23.00");
}

#[tokio::test]
async fn list_with_filters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}/invoices/")))
        .and(query_param("is_cancellation", "false"))
        .and(query_param("order", "ABC12"))
        .and(query_param("order", "XYZ34"))
        .and(query_param("ordering", "-nr"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("invoices/invoice")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/invoices/"))
        .and(query_param("number", "SAMPLECONF-00001"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("invoices/invoice")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let org = client.organizer("bigevents");
    let params = ListInvoices::builder()
        .is_cancellation(false)
        .order(vec!["ABC12".into(), "XYZ34".into()])
        .ordering(Ordering::desc(InvoiceOrdering::Number))
        .build();
    let page = org
        .event("sampleconf")
        .invoices()
        .list(&params)
        .await
        .unwrap();
    assert_eq!(page.results[0].order, "ABC12");
    let all = org
        .invoices()
        .list(
            &ListInvoices::builder()
                .number(vec!["SAMPLECONF-00001".into()])
                .build(),
        )
        .all()
        .await
        .unwrap();
    assert_eq!(all.len(), 1);
}

#[tokio::test]
async fn get_download_and_actions() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}/invoices/SAMPLECONF-00001/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("invoices/invoice")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}/invoices/SAMPLECONF-00001/download/")))
        .respond_with(ResponseTemplate::new(409))
        .expect(1)
        .mount(&server)
        .await;
    for action in ["reissue", "regenerate", "transmit", "retransmit"] {
        Mock::given(method("POST"))
            .and(path(format!("{EVENT}/invoices/SAMPLECONF-00001/{action}/")))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
    }

    let invoices = client.organizer("bigevents").event("sampleconf").invoices();
    let invoice = invoices.get("SAMPLECONF-00001").await.unwrap();
    assert_eq!(invoice.number, "SAMPLECONF-00001");
    let status = invoices.download("SAMPLECONF-00001").await.unwrap();
    assert!(status.is_pending());
    invoices.reissue("SAMPLECONF-00001").await.unwrap();
    invoices.regenerate("SAMPLECONF-00001").await.unwrap();
    invoices.transmit("SAMPLECONF-00001").await.unwrap();
    invoices.retransmit("SAMPLECONF-00001").await.unwrap();
}
