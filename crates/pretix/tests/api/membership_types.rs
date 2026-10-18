use pretix::ListParams;
use pretix::resources::membership_types::{
    CreateMembershipType, MembershipType, UpdateMembershipType,
};
use pretix::types::LocalizedString;
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const TYPES: &str = "/api/v1/organizers/bigevents/membershiptypes/";

#[test]
fn membership_type_model() {
    assert_model::<MembershipType>(&fixture("membership_types/membership_type"), &[]);
}

#[tokio::test]
async fn crud() {
    let (server, client) = setup().await;
    let membership_type = fixture("membership_types/membership_type");
    Mock::given(method("GET"))
        .and(path(TYPES))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![membership_type.clone()])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{TYPES}2/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&membership_type))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(TYPES))
        .and(body_json(json!({
            "name": {"de": "Wochenkarte", "en": "Week pass"},
            "transferable": false,
            "allow_parallel_usage": false,
            "max_usages": 7,
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&membership_type))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{TYPES}2/")))
        .and(body_json(json!({"max_usages": 3})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&membership_type))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{TYPES}2/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let types = client.organizer("bigevents").membership_types();
    let page = types.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].name.localize("en"), Some("Week pass"));
    assert_eq!(types.get(2).await.unwrap().max_usages, Some(7));
    let body = CreateMembershipType::builder()
        .name(LocalizedString::from([
            ("de", "Wochenkarte"),
            ("en", "Week pass"),
        ]))
        .transferable(false)
        .allow_parallel_usage(false)
        .max_usages(7)
        .build();
    types.create(&body).await.unwrap();
    types
        .update(
            2,
            &UpdateMembershipType::builder().max_usages(Some(3)).build(),
        )
        .await
        .unwrap();
    types.delete(2).await.unwrap();
}
