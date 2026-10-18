use pretix::ListParams;
use pretix::resources::event_meta_properties::{
    CreateEventMetaProperty, EventMetaProperty, EventMetaPropertyChoice, UpdateEventMetaProperty,
};
use pretix::types::LocalizedString;
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn event_meta_property_models() {
    let listed = assert_model::<EventMetaProperty>(
        &fixture("event_meta_properties/event_meta_property_in_list"),
        &[],
    );
    assert_eq!(listed.choices.unwrap()[0].key, "blue");
    assert_model::<EventMetaProperty>(&fixture("event_meta_properties/event_meta_property"), &[]);
    let created = assert_model::<EventMetaProperty>(
        &fixture("event_meta_properties/event_meta_property_created"),
        &[],
    );
    assert_eq!(created.public_label, None);
}

#[tokio::test]
async fn crud() {
    let (server, client) = setup().await;
    let base = "/api/v1/organizers/bigevents/event_meta_properties/";
    Mock::given(method("GET"))
        .and(path(base))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture(
                "event_meta_properties/event_meta_property_in_list",
            )])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}1/")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixture("event_meta_properties/event_meta_property")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(base))
        .and(body_json(json!({
            "name": "ref-code",
            "default": "abcde",
            "required": true,
            "choices": [{"key": "abcde"}, {"key": "fghij", "label": {"en": "FGHIJ"}}],
        })))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(fixture("event_meta_properties/event_meta_property_created")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{base}2/")))
        .and(body_json(json!({"required": false, "choices": null})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixture("event_meta_properties/event_meta_property_created")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{base}1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let properties = client.organizer("bigevents").event_meta_properties();
    let page = properties.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].name, "Color");
    properties.get(1).await.unwrap();

    let body = CreateEventMetaProperty::builder()
        .name("ref-code")
        .default("abcde")
        .required(true)
        .choices(vec![
            EventMetaPropertyChoice::builder().key("abcde").build(),
            EventMetaPropertyChoice::builder()
                .key("fghij")
                .label(LocalizedString::from([("en", "FGHIJ")]))
                .build(),
        ])
        .build();
    properties.create(&body).await.unwrap();

    let update = UpdateEventMetaProperty::builder()
        .required(false)
        .choices(None)
        .build();
    properties.update(2, &update).await.unwrap();
    properties.delete(1).await.unwrap();
}

#[tokio::test]
async fn create_without_choices() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/bigevents/event_meta_properties/"))
        .and(body_json(json!({"name": "ref-code", "choices": null})))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(fixture("event_meta_properties/event_meta_property_created")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = CreateEventMetaProperty::builder().name("ref-code").build();
    client
        .organizer("bigevents")
        .event_meta_properties()
        .create(&body)
        .await
        .unwrap();
}
