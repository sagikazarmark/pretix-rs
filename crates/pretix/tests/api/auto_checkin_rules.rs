use pretix::ListParams;
use pretix::resources::auto_checkin_rules::{
    AutoCheckinMode, AutoCheckinRule, CreateAutoCheckinRule, UpdateAutoCheckinRule,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const RULES: &str = "/api/v1/organizers/bigevents/events/sampleconf/auto_checkin_rules/";

#[test]
fn rule_model() {
    assert_model::<AutoCheckinRule>(&fixture("auto_checkin_rules/rule"), &[]);
}

#[tokio::test]
async fn crud() {
    let (server, client) = setup().await;
    let rule = fixture("auto_checkin_rules/rule");
    Mock::given(method("GET"))
        .and(path(RULES))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_of(vec![rule.clone()])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{RULES}1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&rule))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(RULES))
        .and(body_json(json!({
            "list": 12345,
            "mode": "placed",
            "all_sales_channels": false,
            "limit_sales_channels": ["web"],
            "all_products": false,
            "limit_products": [2, 3],
            "limit_variations": [456],
            "all_payment_methods": true,
            "limit_payment_methods": [],
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&rule))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{RULES}1/")))
        .and(body_json(json!({"mode": "paid", "list": null})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&rule))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{RULES}1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let rules = client
        .organizer("bigevents")
        .event("sampleconf")
        .auto_checkin_rules();
    let page = rules
        .list(&ListParams::builder().page(2).build())
        .await
        .unwrap();
    assert_eq!(page.results[0].mode, AutoCheckinMode::Placed);
    assert_eq!(rules.get(1).await.unwrap().list, Some(12345));

    let body = CreateAutoCheckinRule::builder()
        .list(12345)
        .mode(AutoCheckinMode::Placed)
        .all_sales_channels(false)
        .limit_sales_channels(vec!["web".into()])
        .all_products(false)
        .limit_products(vec![2, 3])
        .limit_variations(vec![456])
        .all_payment_methods(true)
        .limit_payment_methods(vec![])
        .build();
    rules.create(&body).await.unwrap();
    let update = UpdateAutoCheckinRule::builder()
        .mode(AutoCheckinMode::Paid)
        .list(None)
        .build();
    rules.update(1, &update).await.unwrap();
    rules.delete(1).await.unwrap();
}
