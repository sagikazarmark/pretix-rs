use jiff::Timestamp;
use pretix::ListParams;
use pretix::query::Ordering;
use pretix::resources::items::{
    AvailabilityMode, CreateItem, CreateItemAddOn, CreateItemBundle, CreateItemProgramTime,
    CreateItemVariation, Item, ItemAddOn, ItemBundle, ItemOrdering, ItemProgramTime, ItemVariation,
    ListItemVariations, ListItems, UpdateItem, UpdateItemAddOn, UpdateItemBundle,
    UpdateItemProgramTime, UpdateItemVariation,
};
use pretix::types::LocalizedString;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const ITEMS: &str = "/api/v1/organizers/bigevents/events/sampleconf/items/";

#[test]
fn item_models() {
    let item = assert_model::<Item>(&fixture("items/item"), &[]);
    assert_eq!(item.variations.len(), 2);
    assert_eq!(item.variations[0].id, 1);
    assert_eq!(item.hidden_if_item_available_mode, AvailabilityMode::Hide);
    let mut legacy = fixture("items/item");
    legacy
        .as_object_mut()
        .unwrap()
        .remove("hidden_if_item_available_mode");
    let legacy: Item = serde_json::from_value(legacy).unwrap();
    assert_eq!(legacy.hidden_if_item_available_mode, AvailabilityMode::Hide);
    let created = assert_model::<Item>(&fixture("items/item_created"), &[]);
    assert_eq!(created.program_times[0].id, None);
}

#[test]
fn nested_models() {
    let variation = assert_model::<ItemVariation>(&fixture("items/variation"), &[]);
    assert_eq!(variation.id, 3);
    assert_eq!(variation.available_from_mode, AvailabilityMode::Hide);
    let listed = assert_model::<ItemVariation>(&fixture("items/variation_in_list"), &[]);
    assert_eq!(listed.price.to_string(), "223");
    assert_model::<ItemBundle>(&fixture("items/bundle"), &[]);
    assert_model::<ItemAddOn>(&fixture("items/addon"), &[]);
    assert_model::<ItemProgramTime>(&fixture("items/program_time"), &[]);
    assert_model::<ItemProgramTime>(&fixture("items/program_time_updated"), &[]);
}

#[tokio::test]
async fn list_with_filters_and_get() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(ITEMS))
        .and(query_param("search", "ticket"))
        .and(query_param("category", "3"))
        .and(query_param("tax_rate", "19.00"))
        .and(query_param("ordering", "position"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("items/item")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{ITEMS}1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("items/item")))
        .expect(1)
        .mount(&server)
        .await;

    let items = client.organizer("bigevents").event("sampleconf").items();
    let params = ListItems::builder()
        .search("ticket")
        .category(3)
        .tax_rate("19.00".parse().unwrap())
        .ordering(Ordering::asc(ItemOrdering::Position))
        .build();
    let page = items.list(&params).await.unwrap();
    assert_eq!(page.results[0].default_price.to_string(), "23.00");
    let item = items.get(1).await.unwrap();
    assert_eq!(item.limit_sales_channels, vec!["web".to_owned()]);
}

#[tokio::test]
async fn create_update_delete() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(ITEMS))
        .and(body_json(json!({
            "name": {"en": "Standard ticket"},
            "default_price": "23.00",
            "tax_rule": 1,
            "variations": [{"value": {"en": "Student"}, "default_price": "10.00"}],
            "addons": [{"addon_category": 1, "max_count": 10}],
            "bundles": [{"bundled_item": 3, "count": 2}],
            "program_times": [{
                "start": "2025-08-14T22:00:00Z",
                "end": "2025-08-15T00:00:00Z",
            }],
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("items/item_created")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{ITEMS}1/")))
        .and(body_json(json!({
            "name": {"en": "Ticket"},
            "default_price": "25.00",
            "category": null,
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("items/item")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{ITEMS}1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let items = client.organizer("bigevents").event("sampleconf").items();
    let body = CreateItem::builder()
        .name(LocalizedString::from([("en", "Standard ticket")]))
        .default_price("23.00".parse().unwrap())
        .tax_rule(1)
        .variations(vec![
            CreateItemVariation::builder()
                .value(LocalizedString::from([("en", "Student")]))
                .default_price("10.00".parse().unwrap())
                .build(),
        ])
        .addons(vec![
            CreateItemAddOn::builder()
                .addon_category(1)
                .max_count(10)
                .build(),
        ])
        .bundles(vec![
            CreateItemBundle::builder().bundled_item(3).count(2).build(),
        ])
        .program_times(vec![
            CreateItemProgramTime::builder()
                .start("2025-08-14T22:00:00Z".parse().unwrap())
                .end("2025-08-15T00:00:00Z".parse().unwrap())
                .build(),
        ])
        .build();
    let item = items.create(&body).await.unwrap();
    assert!(item.has_variations);

    let update = UpdateItem::builder()
        .name(LocalizedString::from([("en", "Ticket")]))
        .default_price("25.00".parse().unwrap())
        .category(None)
        .build();
    items.update(1, &update).await.unwrap();
    items.delete(1).await.unwrap();
}

#[tokio::test]
async fn variations() {
    let (server, client) = setup().await;
    let base = format!("{ITEMS}1/variations/");
    Mock::given(method("GET"))
        .and(path(&base))
        .and(query_param("active", "true"))
        .and(query_param("search", "S"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("items/variation_in_list")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}3/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("items/variation")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(&base))
        .and(body_json(json!({
            "value": {"en": "Student"},
            "default_price": "10.00",
            "available_from_mode": "info",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("items/variation")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{base}3/")))
        .and(body_json(
            json!({"active": false, "position": 1, "default_price": null}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("items/variation")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{base}3/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let variations = client
        .organizer("bigevents")
        .event("sampleconf")
        .items()
        .variations(1);
    let params = ListItemVariations::builder()
        .active(true)
        .search("S")
        .build();
    let page = variations.list(&params).await.unwrap();
    assert_eq!(page.results[0].id, 1);
    variations.get(3).await.unwrap();
    let body = CreateItemVariation::builder()
        .value(LocalizedString::from([("en", "Student")]))
        .default_price("10.00".parse().unwrap())
        .available_from_mode(AvailabilityMode::Info)
        .build();
    variations.create(&body).await.unwrap();
    let update = UpdateItemVariation::builder()
        .active(false)
        .position(1)
        .default_price(None)
        .build();
    variations.update(3, &update).await.unwrap();
    variations.delete(3).await.unwrap();
}

#[tokio::test]
async fn bundles() {
    let (server, client) = setup().await;
    let base = format!("{ITEMS}1/bundles/");
    Mock::given(method("GET"))
        .and(path(&base))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("items/bundle")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}3/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("items/bundle")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(&base))
        .and(body_json(json!({
            "bundled_item": 3,
            "count": 2,
            "designated_price": "1.50",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("items/bundle")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{base}3/")))
        .and(body_json(json!({"bundled_variation": null, "count": 2})))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("items/bundle")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{base}3/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let bundles = client
        .organizer("bigevents")
        .event("sampleconf")
        .items()
        .bundles(1);
    let page = bundles.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].count, 2);
    bundles.get(3).await.unwrap();
    let body = CreateItemBundle::builder()
        .bundled_item(3)
        .count(2)
        .designated_price("1.50".parse().unwrap())
        .build();
    bundles.create(&body).await.unwrap();
    let update = UpdateItemBundle::builder()
        .bundled_variation(None)
        .count(2)
        .build();
    bundles.update(3, &update).await.unwrap();
    bundles.delete(3).await.unwrap();
}

#[tokio::test]
async fn addons() {
    let (server, client) = setup().await;
    let base = format!("{ITEMS}1/addons/");
    Mock::given(method("GET"))
        .and(path(&base))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("items/addon")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}3/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("items/addon")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(&base))
        .and(body_json(json!({
            "addon_category": 1,
            "min_count": 0,
            "max_count": 10,
            "position": 1,
            "multi_allowed": false,
            "price_included": true,
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("items/addon")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{base}3/")))
        .and(body_json(json!({"min_count": 0, "max_count": 10})))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("items/addon")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{base}3/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let addons = client
        .organizer("bigevents")
        .event("sampleconf")
        .items()
        .addons(1);
    let page = addons.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].max_count, 10);
    addons.get(3).await.unwrap();
    let body = CreateItemAddOn::builder()
        .addon_category(1)
        .min_count(0)
        .max_count(10)
        .position(1)
        .multi_allowed(false)
        .price_included(true)
        .build();
    addons.create(&body).await.unwrap();
    let update = UpdateItemAddOn::builder()
        .min_count(0)
        .max_count(10)
        .build();
    addons.update(3, &update).await.unwrap();
    addons.delete(3).await.unwrap();
}

#[tokio::test]
async fn program_times() {
    let (server, client) = setup().await;
    let base = format!("{ITEMS}1/program_times/");
    Mock::given(method("GET"))
        .and(path(&base))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("items/program_time")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("items/program_time")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(&base))
        .and(body_json(json!({
            "start": "2025-08-15T10:00:00Z",
            "end": "2025-08-15T22:00:00Z",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("items/program_time")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{base}1/")))
        .and(body_json(json!({"start": "2025-08-14T10:00:00Z"})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("items/program_time_updated")),
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

    let times = client
        .organizer("bigevents")
        .event("sampleconf")
        .items()
        .program_times(1);
    let page = times.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].id, Some(1));
    times.get(1).await.unwrap();
    let body = CreateItemProgramTime::builder()
        .start("2025-08-15T10:00:00Z".parse().unwrap())
        .end("2025-08-15T22:00:00Z".parse().unwrap())
        .build();
    times.create(&body).await.unwrap();
    let update = UpdateItemProgramTime::builder()
        .start("2025-08-14T10:00:00Z".parse::<Timestamp>().unwrap())
        .build();
    let updated = times.update(1, &update).await.unwrap();
    assert_eq!(updated.location, None);
    times.delete(1).await.unwrap();
}
