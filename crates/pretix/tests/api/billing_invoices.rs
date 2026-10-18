use pretix::JobStatus;
use pretix::query::Ordering;
use pretix::resources::billing_invoices::{
    BillingInvoice, BillingInvoiceOrdering, ListBillingInvoices,
};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn billing_invoice_model() {
    assert_model::<BillingInvoice>(&fixture("billing_invoices/billing_invoice"), &[]);
}

#[tokio::test]
async fn list_and_get() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/billing_invoices/"))
        .and(query_param("ordering", "-date_issued"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("billing_invoices/billing_invoice")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/billing_invoices/R2019002/",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("billing_invoices/billing_invoice")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let invoices = client.organizer("bigevents").billing_invoices();
    let params = ListBillingInvoices::builder()
        .ordering(Ordering::desc(BillingInvoiceOrdering::DateIssued))
        .build();
    let page = invoices.list(&params).await.unwrap();
    assert_eq!(page.results[0].invoice_number, "R2019002");
    let invoice = invoices.get("R2019002").await.unwrap();
    assert_eq!(invoice.date_issued.to_string(), "2019-06-03");
}

#[tokio::test]
async fn download_locked_then_ready() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/billing_invoices/R2019002/download/",
        ))
        .respond_with(ResponseTemplate::new(423))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/billing_invoices/R2019002/download/",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/pdf")
                .insert_header(
                    "content-disposition",
                    r#"attachment; filename="R2019002.pdf""#,
                )
                .set_body_bytes(b"%PDF".to_vec()),
        )
        .expect(1)
        .mount(&server)
        .await;

    let invoices = client.organizer("bigevents").billing_invoices();
    let status = invoices.download("R2019002").await.unwrap();
    assert_eq!(status, JobStatus::Pending { percentage: None });
    let file = invoices
        .download("R2019002")
        .await
        .unwrap()
        .ready()
        .unwrap();
    assert_eq!(file.filename.as_deref(), Some("R2019002.pdf"));
    assert_eq!(&file.data[..], b"%PDF");
}
