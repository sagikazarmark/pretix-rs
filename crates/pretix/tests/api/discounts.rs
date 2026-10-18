use pretix::query::Ordering;
use pretix::resources::discounts::{
    CreateDiscount, Discount, DiscountOrdering, ListDiscounts, SubeventMode, UpdateDiscount,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn discount_model() {
    let discount = assert_model::<Discount>(&fixture("discounts/discount"), &[]);
    assert_eq!(discount.subevent_mode, SubeventMode::Mixed);
    assert_eq!(
        discount.benefit_discount_matching_percent.to_string(),
        "100.00"
    );
}

#[tokio::test]
async fn list_with_filters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/discounts/",
        ))
        .and(query_param("active", "true"))
        .and(query_param("ordering", "-position"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("discounts/discount")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListDiscounts::builder()
        .active(true)
        .ordering(Ordering::desc(DiscountOrdering::Position))
        .build();
    let page = client
        .organizer("bigevents")
        .event("sampleconf")
        .discounts()
        .list(&params)
        .await
        .unwrap();
    assert_eq!(page.results[0].internal_name, "3 for 2");
}

#[tokio::test]
async fn create_get_update_delete() {
    let (server, client) = setup().await;
    let discount = fixture("discounts/discount");
    let base = "/api/v1/organizers/bigevents/events/sampleconf/discounts/";
    Mock::given(method("POST"))
        .and(path(base))
        .and(body_json(json!({
            "internal_name": "3 for 2",
            "subevent_mode": "mixed",
            "condition_min_count": 3,
            "condition_min_value": "0.00",
            "benefit_discount_matching_percent": "100.00",
            "benefit_only_apply_to_cheapest_n_matches": 1,
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&discount))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&discount))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{base}1/")))
        .and(body_json(json!({"active": false, "available_until": null})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&discount))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{base}1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let discounts = client
        .organizer("bigevents")
        .event("sampleconf")
        .discounts();
    let body = CreateDiscount::builder()
        .internal_name("3 for 2")
        .subevent_mode(SubeventMode::Mixed)
        .condition_min_count(3)
        .condition_min_value("0.00".parse().unwrap())
        .benefit_discount_matching_percent("100.00".parse().unwrap())
        .benefit_only_apply_to_cheapest_n_matches(1)
        .build();
    discounts.create(&body).await.unwrap();
    discounts.get(1).await.unwrap();
    let update = UpdateDiscount::builder()
        .active(false)
        .available_until(None)
        .build();
    discounts.update(1, &update).await.unwrap();
    discounts.delete(1).await.unwrap();
}
