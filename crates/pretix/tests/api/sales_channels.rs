use pretix::ListParams;
use pretix::resources::sales_channels::{
    CreateSalesChannel, SalesChannel, SalesChannelType, UpdateSalesChannel,
};
use pretix::types::LocalizedString;
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn sales_channel_models() {
    let channel = assert_model::<SalesChannel>(&fixture("sales_channels/sales_channel"), &[]);
    assert_eq!(channel.r#type, SalesChannelType::Web);
    let channel = assert_model::<SalesChannel>(&fixture("sales_channels/sales_channel_api"), &[]);
    assert_eq!(channel.r#type, SalesChannelType::Api);
}

#[tokio::test]
async fn list_and_get() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/saleschannels/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("sales_channels/sales_channel")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/saleschannels/web/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("sales_channels/sales_channel")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let channels = client.organizer("bigevents").sales_channels();
    let page = channels.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].identifier, "web");
    channels.get("web").await.unwrap();
}

#[tokio::test]
async fn create_update_delete() {
    let (server, client) = setup().await;
    let created = fixture("sales_channels/sales_channel_api");
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/bigevents/saleschannels/"))
        .and(body_json(json!({
            "identifier": "api.custom",
            "label": {"en": "Custom integration"},
            "type": "api",
            "position": 2,
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&created))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/organizers/bigevents/saleschannels/web/"))
        .and(body_json(json!({"position": 5})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("sales_channels/sales_channel")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(
            "/api/v1/organizers/bigevents/saleschannels/api.custom/",
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let channels = client.organizer("bigevents").sales_channels();
    let body = CreateSalesChannel::builder()
        .identifier("api.custom")
        .label(LocalizedString::from([("en", "Custom integration")]))
        .r#type(SalesChannelType::Api)
        .position(2)
        .build();
    let channel = channels.create(&body).await.unwrap();
    assert_eq!(channel.identifier, "api.custom");
    channels
        .update("web", &UpdateSalesChannel::builder().position(5).build())
        .await
        .unwrap();
    channels.delete("api.custom").await.unwrap();
}
