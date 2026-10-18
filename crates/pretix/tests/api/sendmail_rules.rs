use pretix::ListParams;
use pretix::resources::sendmail_rules::{
    CheckedInStatus, CreateSendmailRule, SendTo, SendmailOrderStatus, SendmailRule,
    UpdateSendmailRule,
};
use pretix::types::LocalizedString;
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const BASE: &str = "/api/v1/organizers/bigevents/events/sampleconf/sendmail_rules/";

#[test]
fn sendmail_rule_models() {
    let rule = assert_model::<SendmailRule>(&fixture("sendmail_rules/sendmail_rule"), &[]);
    assert_eq!(rule.checked_in_status, None);
    assert_eq!(rule.send_offset_time, Some("18:00".parse().unwrap()));
    assert_eq!(
        rule.restrict_to_status[1],
        SendmailOrderStatus::PaymentPending
    );
    let rule =
        assert_model::<SendmailRule>(&fixture("sendmail_rules/sendmail_rule_checked_in"), &[]);
    assert_eq!(rule.checked_in_status, Some(CheckedInStatus::CheckedIn));
}

#[tokio::test]
async fn rule_crud() {
    let (server, client) = setup().await;
    let rule = fixture("sendmail_rules/sendmail_rule_checked_in");
    Mock::given(method("GET"))
        .and(path(BASE))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("sendmail_rules/sendmail_rule")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{BASE}1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&rule))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(BASE))
        .and(body_json(json!({
            "enabled": true,
            "subject": {"en": "See you tomorrow!"},
            "template": {"en": "Don't forget your tickets, download them at {url}"},
            "all_products": true,
            "restrict_to_status": ["p", "n__valid_if_pending"],
            "checked_in_status": "checked_in",
            "send_offset_days": 1,
            "send_offset_time": "18:00:00",
            "date_is_absolute": false,
            "send_to": "orders",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&rule))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{BASE}1/")))
        .and(body_json(
            json!({"enabled": false, "checked_in_status": null}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&rule))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{BASE}1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let rules = client
        .organizer("bigevents")
        .event("sampleconf")
        .sendmail_rules();
    let page = rules.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].send_to, SendTo::Orders);
    rules.get(1).await.unwrap();

    let body = CreateSendmailRule::builder()
        .enabled(true)
        .subject(LocalizedString::from([("en", "See you tomorrow!")]))
        .template(LocalizedString::from([(
            "en",
            "Don't forget your tickets, download them at {url}",
        )]))
        .all_products(true)
        .restrict_to_status(vec![
            SendmailOrderStatus::Paid,
            SendmailOrderStatus::ValidIfPending,
        ])
        .checked_in_status(CheckedInStatus::CheckedIn)
        .send_offset_days(1)
        .send_offset_time("18:00".parse().unwrap())
        .date_is_absolute(false)
        .send_to(SendTo::Orders)
        .build();
    rules.create(&body).await.unwrap();

    let update = UpdateSendmailRule::builder()
        .enabled(false)
        .checked_in_status(None)
        .build();
    rules.update(1, &update).await.unwrap();
    rules.delete(1).await.unwrap();
}
