use pretix::ListParams;
use pretix::resources::digital_contents::{
    CreateDigitalContent, DigitalContent, DigitalContentType, UpdateDigitalContent,
};
use pretix::types::LocalizedString;
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const CONTENTS: &str = "/api/v1/organizers/bigevents/events/sampleconf/digitalcontents/";

#[test]
fn digital_content_model() {
    let content = assert_model::<DigitalContent>(&fixture("digital_contents/digital_content"), &[]);
    assert_eq!(content.content_type, DigitalContentType::Link);
}

#[tokio::test]
async fn list_and_get() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(CONTENTS))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("digital_contents/digital_content")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{CONTENTS}1/")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("digital_contents/digital_content")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let contents = client
        .organizer("bigevents")
        .event("sampleconf")
        .digital_contents();
    let all = contents.list(&ListParams::default()).all().await.unwrap();
    assert_eq!(all.len(), 1);
    let content = contents.get(1).await.unwrap();
    assert_eq!(content.position, 1);
}

#[tokio::test]
async fn create_update_delete() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(CONTENTS))
        .and(body_json(json!({
            "title": {"en": "Concert livestream"},
            "content_type": "link",
            "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            "description": {"en": "Watch our event live here on YouTube!"},
            "all_products": true,
            "limit_products": [],
            "available_from": "2020-03-22T23:00:00Z",
            "position": 1,
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(fixture("digital_contents/digital_content")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{CONTENTS}1/")))
        .and(body_json(
            json!({"url": "https://mywebsite.com", "available_until": null}),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("digital_contents/digital_content")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{CONTENTS}1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let contents = client
        .organizer("bigevents")
        .event("sampleconf")
        .digital_contents();
    let body = CreateDigitalContent::builder()
        .title(LocalizedString::from([("en", "Concert livestream")]))
        .content_type(DigitalContentType::Link)
        .url("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
        .description(LocalizedString::from([(
            "en",
            "Watch our event live here on YouTube!",
        )]))
        .all_products(true)
        .limit_products(vec![])
        .available_from("2020-03-22T23:00:00Z".parse().unwrap())
        .position(1)
        .build();
    contents.create(&body).await.unwrap();
    let update = UpdateDigitalContent::builder()
        .url(Some("https://mywebsite.com".into()))
        .available_until(None)
        .build();
    contents.update(1, &update).await.unwrap();
    contents.delete(1).await.unwrap();
}
