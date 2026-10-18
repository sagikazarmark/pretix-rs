use pretix::ListParams;
use pretix::resources::campaigns::{Campaign, CreateCampaign, UpdateCampaign};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn campaign_model() {
    assert_model::<Campaign>(&fixture("campaigns/campaign"), &[]);
}

#[tokio::test]
async fn list_and_get() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/campaigns/",
        ))
        .and(query_param("page", "2"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("campaigns/campaign")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/campaigns/1/",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("campaigns/campaign")))
        .expect(1)
        .mount(&server)
        .await;

    let campaigns = client
        .organizer("bigevents")
        .event("sampleconf")
        .campaigns();
    let page = campaigns
        .list(&ListParams::builder().page(2).build())
        .await
        .unwrap();
    assert_eq!(page.results[0].code, "wZnL11fjq");
    let campaign = campaigns.get(1).await.unwrap();
    assert_eq!(campaign.description, "Facebook");
}

#[tokio::test]
async fn create_update_delete() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/campaigns/",
        ))
        .and(body_json(json!({"description": "Twitter"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("campaigns/campaign")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/campaigns/1/",
        ))
        .and(body_json(
            json!({"external_target": "https://mywebsite.com"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("campaigns/campaign")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/campaigns/1/",
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let campaigns = client
        .organizer("bigevents")
        .event("sampleconf")
        .campaigns();
    campaigns
        .create(&CreateCampaign::builder().description("Twitter").build())
        .await
        .unwrap();
    let update = UpdateCampaign::builder()
        .external_target("https://mywebsite.com")
        .build();
    campaigns.update(1, &update).await.unwrap();
    campaigns.delete(1).await.unwrap();
}
