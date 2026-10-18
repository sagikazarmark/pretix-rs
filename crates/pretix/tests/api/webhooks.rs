use pretix::resources::webhooks::{CreateWebhook, ListWebhooks, UpdateWebhook, Webhook};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn webhook_model() {
    assert_model::<Webhook>(&fixture("webhooks/webhook"), &[]);
}

#[tokio::test]
async fn webhook_crud() {
    let (server, client) = setup().await;
    let webhook = fixture("webhooks/webhook");
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/webhooks/"))
        .and(query_param("enabled", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_of(vec![webhook.clone()])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/webhooks/2/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&webhook))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/bigevents/webhooks/"))
        .and(body_json(json!({
            "enabled": true,
            "target_url": "https://httpstat.us/200",
            "all_events": false,
            "limit_events": ["democon"],
            "action_types": ["pretix.event.order.modified", "pretix.event.order.changed.*"],
            "comment": "Called for changes",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&webhook))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/organizers/bigevents/webhooks/2/"))
        .and(body_json(json!({"enabled": false, "comment": null})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&webhook))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/organizers/bigevents/webhooks/2/"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let webhooks = client.organizer("bigevents").webhooks();
    let page = webhooks
        .list(&ListWebhooks::builder().enabled(true).build())
        .await
        .unwrap();
    assert_eq!(page.results[0].limit_events, ["democon"]);
    assert_eq!(webhooks.get(2).await.unwrap().id, 2);

    let body = CreateWebhook::builder()
        .enabled(true)
        .target_url("https://httpstat.us/200")
        .all_events(false)
        .limit_events(vec!["democon".into()])
        .action_types(vec![
            "pretix.event.order.modified".into(),
            "pretix.event.order.changed.*".into(),
        ])
        .comment("Called for changes")
        .build();
    webhooks.create(&body).await.unwrap();

    let update = UpdateWebhook::builder()
        .enabled(false)
        .comment(None)
        .build();
    webhooks.update(2, &update).await.unwrap();
    webhooks.delete(2).await.unwrap();
}
