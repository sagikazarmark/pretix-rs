use pretix::query::Ordering;
use pretix::resources::categories::{
    Category, CategoryOrdering, CreateCategory, CrossSellingMode, ListCategories, UpdateCategory,
};
use pretix::types::LocalizedString;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn category_model() {
    assert_model::<Category>(&fixture("categories/category"), &[]);
}

#[tokio::test]
async fn list_and_get() {
    let (server, client) = setup().await;
    let base = "/api/v1/organizers/bigevents/events/sampleconf/categories/";
    Mock::given(method("GET"))
        .and(path(base))
        .and(query_param("is_addon", "false"))
        .and(query_param("ordering", "-id"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("categories/category")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("categories/category")))
        .expect(1)
        .mount(&server)
        .await;

    let categories = client
        .organizer("bigevents")
        .event("sampleconf")
        .categories();
    let params = ListCategories::builder()
        .is_addon(false)
        .ordering(Ordering::desc(CategoryOrdering::Id))
        .build();
    let page = categories.list(&params).await.unwrap();
    assert_eq!(page.results[0].name.localize("en"), Some("Tickets"));
    let category = categories.get(1).await.unwrap();
    assert_eq!(category.position, 1);
}

#[tokio::test]
async fn create_update_delete() {
    let (server, client) = setup().await;
    let base = "/api/v1/organizers/bigevents/events/sampleconf/categories/";
    Mock::given(method("POST"))
        .and(path(base))
        .and(body_json(json!({
            "name": {"en": "Tickets"},
            "position": 1,
            "cross_selling_mode": "both",
            "cross_selling_match_products": [],
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("categories/category")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{base}1/")))
        .and(body_json(
            json!({"is_addon": true, "cross_selling_mode": null}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("categories/category")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{base}1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let categories = client
        .organizer("bigevents")
        .event("sampleconf")
        .categories();
    let body = CreateCategory::builder()
        .name(LocalizedString::from([("en", "Tickets")]))
        .position(1)
        .cross_selling_mode(CrossSellingMode::Both)
        .cross_selling_match_products(vec![])
        .build();
    categories.create(&body).await.unwrap();
    let update = UpdateCategory::builder()
        .is_addon(true)
        .cross_selling_mode(None)
        .build();
    categories.update(1, &update).await.unwrap();
    categories.delete(1).await.unwrap();
}
