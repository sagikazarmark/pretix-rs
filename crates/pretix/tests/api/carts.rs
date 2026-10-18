use pretix::ListParams;
use pretix::resources::carts::{
    BulkCreateResult, CartPosition, CreateCartAddon, CreateCartPosition,
};
use pretix::resources::orders::CreateAnswer;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const BASE: &str = "/api/v1/organizers/bigevents/events/sampleconf/cartpositions";

#[test]
fn cart_position_models() {
    assert_model::<CartPosition>(&fixture("carts/cart_position"), &[]);
    let created = assert_model::<CartPosition>(&fixture("carts/cart_position_created"), &[]);
    assert_eq!(created.addons[0].addon_to, Some(1));
    let results = assert_model::<Vec<BulkCreateResult>>(
        &fixture("carts/bulk_create_response")["results"],
        &[],
    );
    assert!(results[0].success);
    // The docs show `"success": "false"`, which is accepted as `false`.
    assert!(!results[1].success);
    assert!(results[1].data.is_none());
}

fn peter_or(name: &str) -> CreateCartPosition {
    CreateCartPosition::builder()
        .item(1)
        .price("23.00".parse().unwrap())
        .attendee_name_parts(
            [
                ("given_name".to_owned(), name.to_owned()),
                ("family_name".to_owned(), "Miller".to_owned()),
            ]
            .into(),
        )
        .answers(vec![
            CreateAnswer::builder().question(1).answer("23").build(),
        ])
        .build()
}

#[tokio::test]
async fn list_get_create_delete() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(format!("{BASE}/")))
        .and(query_param("page_size", "10"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("carts/cart_position")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{BASE}/1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("carts/cart_position")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{BASE}/")))
        .and(body_json(json!({
            "item": 1,
            "price": "23.00",
            "attendee_name_parts": {"given_name": "Peter", "family_name": "Miller"},
            "answers": [{"question": 1, "answer": "23"}],
            "addons": [{"item": 2, "price": "0.00", "attendee_name": "Peter Miller"}],
            "bundled": [{"item": 3, "variation": 4, "price": "1.50"}]
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(fixture("carts/cart_position_created")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{BASE}/1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let carts = client
        .organizer("bigevents")
        .event("sampleconf")
        .cart_positions();
    let page = carts
        .list(&ListParams::builder().page_size(10).build())
        .await
        .unwrap();
    assert!(page.results[0].cart_id.ends_with("@api"));
    carts.get(1).await.unwrap();
    let mut body = peter_or("Peter");
    body.addons = vec![
        CreateCartAddon::builder()
            .item(2)
            .price("0.00".parse().unwrap())
            .attendee_name("Peter Miller")
            .build(),
    ];
    body.bundled = vec![
        CreateCartAddon::builder()
            .item(3)
            .variation(4)
            .price("1.50".parse().unwrap())
            .build(),
    ];
    let created = carts.create(&body).await.unwrap();
    assert_eq!(created.addons.len(), 1);
    carts.delete(1).await.unwrap();
}

#[tokio::test]
async fn bulk_create() {
    let (server, client) = setup().await;
    let position = |name: &str| {
        json!({
            "item": 1,
            "price": "23.00",
            "attendee_name_parts": {"given_name": name, "family_name": "Miller"},
            "answers": [{"question": 1, "answer": "23"}]
        })
    };
    Mock::given(method("POST"))
        .and(path(format!("{BASE}/bulk_create/")))
        .and(body_json(json!([position("Peter"), position("Maria")])))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("carts/bulk_create_response")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let results = client
        .organizer("bigevents")
        .event("sampleconf")
        .cart_positions()
        .bulk_create(&[peter_or("Peter"), peter_or("Maria")])
        .await
        .unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].data.as_ref().unwrap().id, 1);
    assert!(!results[1].success);
    assert!(results[1].errors.as_ref().unwrap()["non_field_errors"].is_array());
}
