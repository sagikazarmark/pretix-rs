use pretix::ListParams;
use pretix::resources::exhibitors::{
    AttachVoucher, CreateExhibitor, Exhibitor, ExhibitorVoucher, Lead, UpdateExhibitor,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const EXHIBITORS: &str = "/api/v1/organizers/bigevents/events/sampleconf/exhibitors/";

#[test]
fn exhibitor_models() {
    let exhibitor = assert_model::<Exhibitor>(&fixture("exhibitors/exhibitor"), &[]);
    assert!(!exhibitor.lead_scanning_scope_by_device);
    let lead = assert_model::<Lead>(&fixture("exhibitors/lead"), &[]);
    assert_eq!(lead.data[0].details.len(), 2);
    assert_eq!(lead.device_name.as_deref(), Some("DEV1"));
    let voucher = assert_model::<ExhibitorVoucher>(&fixture("exhibitors/voucher"), &[]);
    assert_eq!(voucher.voucher.code, "43K6LKM37FBVR2YG");
}

#[tokio::test]
async fn list_get_leads_vouchers() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(EXHIBITORS))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("exhibitors/exhibitor")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{EXHIBITORS}1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("exhibitors/exhibitor")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{EXHIBITORS}1/leads/")))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("exhibitors/lead")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{EXHIBITORS}1/vouchers/")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("exhibitors/voucher")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let exhibitors = client
        .organizer("bigevents")
        .event("sampleconf")
        .exhibitors();
    let page = exhibitors.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].name, "Aperture Science");
    let exhibitor = exhibitors.get(1).await.unwrap();
    assert_eq!(exhibitor.booth.as_deref(), Some("A2"));
    let leads = exhibitors
        .leads(1, &ListParams::builder().page(1).build())
        .await
        .unwrap();
    assert_eq!(leads.results[0].attendee_order, "T0E7E");
    let vouchers = exhibitors
        .vouchers(1)
        .list(&ListParams::default())
        .await
        .unwrap();
    assert_eq!(vouchers.results[0].exhibitor_comment, None);
}

#[tokio::test]
async fn create_update_delete() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(EXHIBITORS))
        .and(body_json(json!({
            "name": "Aperture Science",
            "contact_email": "johnson@as.example.org",
            "booth": "A2",
            "allow_lead_scanning": true,
            "exhibitor_tags": ["Gold Sponsor"],
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("exhibitors/exhibitor")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{EXHIBITORS}1/")))
        .and(body_json(json!({"internal_id": "ABC", "booth": null})))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("exhibitors/exhibitor")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{EXHIBITORS}1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let exhibitors = client
        .organizer("bigevents")
        .event("sampleconf")
        .exhibitors();
    let body = CreateExhibitor::builder()
        .name("Aperture Science")
        .contact_email("johnson@as.example.org")
        .booth("A2")
        .allow_lead_scanning(true)
        .exhibitor_tags(vec!["Gold Sponsor".into()])
        .build();
    exhibitors.create(&body).await.unwrap();
    let update = UpdateExhibitor::builder()
        .internal_id(Some("ABC".into()))
        .booth(None)
        .build();
    exhibitors.update(1, &update).await.unwrap();
    exhibitors.delete(1).await.unwrap();
}

#[tokio::test]
async fn actions() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(format!("{EXHIBITORS}1/send_access_code/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{EXHIBITORS}1/vouchers/attach/")))
        .and(body_json(
            json!({"code": "43K6LKM37FBVR2YG", "exhibitor_comment": "Free ticket"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{EXHIBITORS}1/vouchers/bulk_attach/")))
        .and(body_json(json!([
            {"id": 15, "exhibitor_comment": "Free ticket"},
            {"id": 16},
        ])))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let exhibitors = client
        .organizer("bigevents")
        .event("sampleconf")
        .exhibitors();
    exhibitors.send_access_code(1).await.unwrap();
    let attach = AttachVoucher::builder()
        .code("43K6LKM37FBVR2YG")
        .exhibitor_comment("Free ticket")
        .build();
    exhibitors.vouchers(1).attach(&attach).await.unwrap();
    let bulk = [
        AttachVoucher::builder()
            .id(15)
            .exhibitor_comment("Free ticket")
            .build(),
        AttachVoucher::builder().id(16).build(),
    ];
    exhibitors.vouchers(1).bulk_attach(&bulk).await.unwrap();
}
