use pretix::ListParams;
use pretix::resources::resellers::{CreateOrganizer, Reseller, ResellerOrganizer};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn reseller_models() {
    assert_model::<Reseller>(&fixture("resellers/reseller"), &[]);
    assert_model::<ResellerOrganizer>(&fixture("resellers/organizer"), &[]);
}

#[tokio::test]
async fn list_get_create_organizer() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/var/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("resellers/reseller")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/var/1/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("resellers/reseller")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/var/1/create_organizer/"))
        .and(body_json(
            json!({"name": "My new client", "slug": "New client"}),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("resellers/organizer")))
        .expect(1)
        .mount(&server)
        .await;

    let resellers = client.resellers();
    let page = resellers.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].public_name, "ticketshop.live");
    let reseller = resellers.get(1).await.unwrap();
    assert_eq!(reseller.communication_language, "de");
    let body = CreateOrganizer::builder()
        .name("My new client")
        .slug("New client")
        .build();
    let organizer = resellers.create_organizer(1, &body).await.unwrap();
    assert_eq!(organizer.id, 1);
}
