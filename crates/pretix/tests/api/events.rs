use jiff::Timestamp;
use pretix::query::AttrFilters;
use pretix::resources::events::{CreateEvent, Event, ListEvents, UpdateEvent};
use pretix::types::LocalizedString;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn event_models() {
    assert_model::<Event>(&fixture("events/event"), &[]);
    assert_model::<Event>(&fixture("events/event_in_list"), &[]);
}

#[tokio::test]
async fn list_with_filters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/events/"))
        .and(query_param("live", "true"))
        .and(query_param("attr[Format]", "Seminar"))
        .and(query_param("date_from_after", "2017-12-01T00:00:00Z"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("events/event_in_list")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListEvents::builder()
        .live(true)
        .date_from_after("2017-12-01T00:00:00Z".parse::<Timestamp>().unwrap())
        .attr(AttrFilters::default().with("Format", "Seminar"))
        .build();
    let events = client
        .organizer("bigevents")
        .events()
        .list(&params)
        .all()
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].name.localize("en"), Some("Sample Conference"));
}

#[tokio::test]
async fn create_clone_update_delete() {
    let (server, client) = setup().await;
    let created = fixture("events/event");
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/bigevents/events/"))
        .and(query_param("clone_from", "other/template"))
        .and(body_json(json!({
            "name": {"en": "Sample Conference"},
            "slug": "sampleconf",
            "currency": "EUR",
            "date_from": "2017-12-27T10:00:00Z",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&created))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/organizers/bigevents/events/sampleconf/"))
        .and(body_json(json!({"presale_end": null, "live": true})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&created))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/organizers/bigevents/events/sampleconf/"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let events = client.organizer("bigevents").events();
    let body = CreateEvent::builder()
        .name(LocalizedString::from([("en", "Sample Conference")]))
        .slug("sampleconf")
        .currency("EUR")
        .date_from("2017-12-27T10:00:00Z".parse().unwrap())
        .build();
    let event = events.create_from("other/template", &body).await.unwrap();
    assert_eq!(event.slug, "sampleconf");

    let update = UpdateEvent::builder().live(true).presale_end(None).build();
    events.update("sampleconf", &update).await.unwrap();
    events.delete("sampleconf").await.unwrap();
}

fn create_body() -> CreateEvent {
    CreateEvent::builder()
        .name(LocalizedString::from([("en", "Sample Conference")]))
        .slug("sampleconf")
        .currency("EUR")
        .date_from("2017-12-27T10:00:00Z".parse().unwrap())
        .is_public(false)
        .build()
}

#[tokio::test]
async fn get_and_create() {
    let (server, client) = setup().await;
    let event = fixture("events/event");
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/events/sampleconf/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&event))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/bigevents/events/"))
        .and(query_param_is_missing("clone_from"))
        .and(body_json(json!({
            "name": {"en": "Sample Conference"},
            "slug": "sampleconf",
            "currency": "EUR",
            "date_from": "2017-12-27T10:00:00Z",
            "is_public": false,
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&event))
        .expect(1)
        .mount(&server)
        .await;

    let events = client.organizer("bigevents").events();
    let fetched = events.get("sampleconf").await.unwrap();
    assert!(!fetched.is_public);
    let created = events.create(&create_body()).await.unwrap();
    assert_eq!(created.slug, "sampleconf");
}

#[tokio::test]
async fn clone_event() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/bigevents/events/template/clone/"))
        .and(body_json(json!({
            "name": {"en": "Sample Conference"},
            "slug": "sampleconf",
            "currency": "EUR",
            "date_from": "2017-12-27T10:00:00Z",
            "is_public": false,
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("events/event")))
        .expect(1)
        .mount(&server)
        .await;

    let event = client
        .organizer("bigevents")
        .events()
        .clone_event("template", &create_body())
        .await
        .unwrap();
    assert_eq!(event.slug, "sampleconf");
}

#[tokio::test]
async fn settings() {
    let (server, client) = setup().await;
    let settings_path = "/api/v1/organizers/bigevents/events/sampleconf/settings/";
    Mock::given(method("GET"))
        .and(path(settings_path))
        .and(query_param_is_missing("explain"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"imprint_url": null})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(settings_path))
        .and(query_param("explain", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "imprint_url": {
                "value": null,
                "label": "Imprint URL",
                "help_text": "This should point e.g. to a part of your website that has your contact details and legal information.",
                "readonly": false,
            },
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(settings_path))
        .and(body_json(json!({"imprint_url": "https://pretix.eu"})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"imprint_url": "https://pretix.eu"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let events = client.organizer("bigevents").events();
    let settings = events.settings("sampleconf").await.unwrap();
    assert!(settings["imprint_url"].is_null());
    let explained = events.settings_explained("sampleconf").await.unwrap();
    assert_eq!(explained["imprint_url"]["label"], "Imprint URL");
    let updated = events
        .update_settings("sampleconf", &json!({"imprint_url": "https://pretix.eu"}))
        .await
        .unwrap();
    assert_eq!(updated["imprint_url"], "https://pretix.eu");
}
