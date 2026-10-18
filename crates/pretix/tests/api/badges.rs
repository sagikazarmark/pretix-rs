use pretix::ListParams;
use pretix::resources::badges::{BadgeItem, BadgeLayout};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const EVENT: &str = "/api/v1/organizers/bigevents/events/democon/";

#[test]
fn badge_models() {
    assert_model::<BadgeLayout>(&fixture("badges/badge_layout"), &[]);
    assert_model::<BadgeItem>(&fixture("badges/badge_item"), &[]);
}

#[tokio::test]
async fn layouts_and_items() {
    let (server, client) = setup().await;
    let layout = fixture("badges/badge_layout");
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}badgelayouts/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_of(vec![layout.clone()])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}badgelayouts/1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&layout))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}badgeitems/")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("badges/badge_item")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let event = client.organizer("bigevents").event("democon");
    let page = event
        .badge_layouts()
        .list(&ListParams::default())
        .await
        .unwrap();
    assert_eq!(page.results[0].name, "Default layout");
    assert_eq!(event.badge_layouts().get(1).await.unwrap().id, 1);
    let items = event
        .badge_items()
        .list(&ListParams::default())
        .await
        .unwrap();
    assert_eq!(items.results[0].item, 3);
}
