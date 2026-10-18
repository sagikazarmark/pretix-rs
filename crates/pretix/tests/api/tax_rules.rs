use pretix::ListParams;
use pretix::resources::tax_rules::{CreateTaxRule, TaxCode, TaxRule, UpdateTaxRule};
use pretix::types::LocalizedString;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn tax_rule_models() {
    let rule = assert_model::<TaxRule>(&fixture("tax_rules/tax_rule"), &[]);
    assert_eq!(rule.code, Some(TaxCode::Standard));
    let updated = assert_model::<TaxRule>(&fixture("tax_rules/tax_rule_updated"), &[]);
    assert!(!updated.default);
}

#[test]
fn tax_code_with_exemption_reason() {
    let code: TaxCode = serde_json::from_value(json!("E/VATEX-EU-132")).unwrap();
    assert_eq!(code, TaxCode::Unrecognized("E/VATEX-EU-132".into()));
}

#[tokio::test]
async fn list_and_get() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/taxrules/",
        ))
        .and(query_param("page", "2"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("tax_rules/tax_rule")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/taxrules/1/",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("tax_rules/tax_rule")))
        .expect(1)
        .mount(&server)
        .await;

    let rules = client
        .organizer("bigevents")
        .event("sampleconf")
        .tax_rules();
    let page = rules
        .list(&ListParams::builder().page(2).build())
        .await
        .unwrap();
    assert_eq!(page.results[0].rate.to_string(), "19.00");
    let rule = rules.get(1).await.unwrap();
    assert_eq!(rule.home_country.as_deref(), Some("DE"));
}

#[tokio::test]
async fn create_update_delete() {
    let (server, client) = setup().await;
    let base = "/api/v1/organizers/bigevents/events/sampleconf/taxrules/";
    Mock::given(method("POST"))
        .and(path(base))
        .and(body_json(json!({
            "name": {"en": "VAT"},
            "rate": "19.00",
            "price_includes_tax": true,
            "eu_reverse_charge": false,
            "home_country": "DE",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("tax_rules/tax_rule")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{base}1/")))
        .and(body_json(json!({"rate": "20.00", "custom_rules": null})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("tax_rules/tax_rule_updated")),
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

    let rules = client
        .organizer("bigevents")
        .event("sampleconf")
        .tax_rules();
    let body = CreateTaxRule::builder()
        .name(LocalizedString::from([("en", "VAT")]))
        .rate("19.00".parse().unwrap())
        .price_includes_tax(true)
        .eu_reverse_charge(false)
        .home_country("DE")
        .build();
    rules.create(&body).await.unwrap();

    let update = UpdateTaxRule::builder()
        .rate("20.00".parse().unwrap())
        .custom_rules(None)
        .build();
    let updated = rules.update(1, &update).await.unwrap();
    assert_eq!(updated.rate.to_string(), "20.00");
    rules.delete(1).await.unwrap();
}
