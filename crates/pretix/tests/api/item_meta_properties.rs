use pretix::ListParams;
use pretix::resources::item_meta_properties::{
    CreateItemMetaProperty, ItemMetaProperty, UpdateItemMetaProperty,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn item_meta_property_models() {
    assert_model::<ItemMetaProperty>(&fixture("item_meta_properties/item_meta_property"), &[]);
    let created = assert_model::<ItemMetaProperty>(
        &fixture("item_meta_properties/item_meta_property_created"),
        &[],
    );
    assert_eq!(created.allowed_values, None);
}

#[tokio::test]
async fn crud() {
    let (server, client) = setup().await;
    let base = "/api/v1/organizers/bigevents/events/sampleconf/item_meta_properties/";
    Mock::given(method("GET"))
        .and(path(base))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture(
                "item_meta_properties/item_meta_property",
            )])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}1/")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixture("item_meta_properties/item_meta_property")),
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
            "allowed_values": null,
        })))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(fixture("item_meta_properties/item_meta_property_created")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{base}2/")))
        .and(body_json(
            json!({"required": false, "allowed_values": null}),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixture("item_meta_properties/item_meta_property_created")),
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

    let properties = client
        .organizer("bigevents")
        .event("sampleconf")
        .item_meta_properties();
    let page = properties.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].name, "Color");
    let property = properties.get(1).await.unwrap();
    assert_eq!(property.allowed_values.unwrap().len(), 3);

    let body = CreateItemMetaProperty::builder()
        .name("ref-code")
        .default("abcde")
        .required(true)
        .build();
    let created = properties.create(&body).await.unwrap();
    assert_eq!(created.id, 2);

    let update = UpdateItemMetaProperty::builder()
        .required(false)
        .allowed_values(None)
        .build();
    properties.update(2, &update).await.unwrap();
    properties.delete(1).await.unwrap();
}
