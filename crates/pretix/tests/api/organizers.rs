use pretix::query::Ordering;
use pretix::resources::organizers::{
    ListOrganizers, Organizer, OrganizerOrdering, UpdateOrganizer,
};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn organizer_model() {
    assert_model::<Organizer>(&fixture("organizers/organizer"), &[]);
}

#[tokio::test]
async fn list() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/"))
        .and(query_param("ordering", "-name"))
        .and(header("authorization", "Token secret"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("organizers/organizer")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListOrganizers::builder()
        .ordering(Ordering::desc(OrganizerOrdering::Name))
        .build();
    let page = client.organizers().list(&params).await.unwrap();
    assert_eq!(page.count, 1);
    assert_eq!(page.results[0].name, "Big Events LLC");
}

#[tokio::test]
async fn update_and_settings() {
    let (server, client) = setup().await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/organizers/bigevents/"))
        .and(body_json(json!({"plugins": ["pretix_seating"]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("organizers/organizer")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/settings/"))
        .and(query_param("explain", "true"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"event_list_type": {"value": "calendar"}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = UpdateOrganizer::builder()
        .plugins(vec!["pretix_seating".into()])
        .build();
    client
        .organizers()
        .update("bigevents", &body)
        .await
        .unwrap();
    let settings = client
        .organizers()
        .settings_explained("bigevents")
        .await
        .unwrap();
    assert_eq!(settings["event_list_type"]["value"], "calendar");
}

#[tokio::test]
async fn get() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("organizers/organizer")))
        .expect(1)
        .mount(&server)
        .await;

    let organizer = client.organizers().get("bigevents").await.unwrap();
    assert_eq!(organizer.name, "Big Events LLC");
}

#[tokio::test]
async fn settings_and_update_settings() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/settings/"))
        .and(query_param_is_missing("explain"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"event_list_type": "calendar"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/organizers/bigevents/settings/"))
        .and(body_json(
            json!({"event_list_type": "list", "locale": null}),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"event_list_type": "list", "locale": "en"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let organizers = client.organizers();
    let settings = organizers.settings("bigevents").await.unwrap();
    assert_eq!(settings["event_list_type"], "calendar");
    let settings = organizers
        .update_settings(
            "bigevents",
            &json!({"event_list_type": "list", "locale": null}),
        )
        .await
        .unwrap();
    assert_eq!(settings["locale"], "en");
}
