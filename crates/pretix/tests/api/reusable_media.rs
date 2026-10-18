use jiff::Timestamp;
use pretix::query::Expand;
use pretix::resources::reusable_media::{
    CreateReusableMedium, ListReusableMedia, LookupMedium, MediumType, ReusableMedium,
    ReusableMediumExpand, UpdateReusableMedium,
};
use pretix::types::Expandable;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const MEDIA: &str = "/api/v1/organizers/bigevents/reusablemedia/";

#[test]
fn medium_models() {
    assert_model::<ReusableMedium>(&fixture("reusable_media/medium"), &[]);
    let medium = assert_model::<ReusableMedium>(&fixture("reusable_media/medium_linked"), &[]);
    assert_eq!(
        medium.linked_orderpositions,
        vec![Expandable::Id(13), Expandable::Id(29)]
    );
}

#[test]
fn expanded_medium() {
    let mut json = fixture("reusable_media/medium");
    let mut gift_card = fixture("gift_cards/gift_card");
    gift_card["owner_ticket"] = fixture("checkin/position");
    json["linked_giftcard"] = gift_card;
    json["customer"] = fixture("customers/customer");
    json["linked_orderpositions"] = json!([fixture("checkin/position")]);
    let medium = assert_model::<ReusableMedium>(&json, &[]);

    let Some(Expandable::Expanded(gift_card)) = medium.linked_giftcard else {
        panic!("gift card not expanded");
    };
    assert!(matches!(
        gift_card.owner_ticket,
        Some(Expandable::Expanded(_))
    ));
    assert!(matches!(medium.customer, Some(Expandable::Expanded(_))));
    assert!(matches!(
        medium.linked_orderpositions[0],
        Expandable::Expanded(_)
    ));
}

#[tokio::test]
async fn list_with_filters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(MEDIA))
        .and(query_param("type", "nfc_uid"))
        .and(query_param("active", "true"))
        .and(query_param("linked_orderpositions", "13"))
        .and(query_param("updated_since", "2021-04-01T00:00:00Z"))
        .and(query_param("expand", "customer"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("reusable_media/medium")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListReusableMedia::builder()
        .r#type(MediumType::NfcUid)
        .active(true)
        .linked_orderpositions(vec![13, 29])
        .updated_since("2021-04-01T00:00:00Z".parse::<Timestamp>().unwrap())
        .expand(vec![ReusableMediumExpand::Customer])
        .build();
    let request = client.organizer("bigevents").reusable_media().list(&params);
    assert!(
        request
            .url()
            .as_str()
            .contains("linked_orderpositions=13&linked_orderpositions=29")
    );
    let page = request.await.unwrap();
    assert_eq!(page.results[0].identifier, "ABCDEFGH");
}

#[tokio::test]
async fn get_lookup_create_update() {
    let (server, client) = setup().await;
    let medium = fixture("reusable_media/medium");
    Mock::given(method("GET"))
        .and(path(format!("{MEDIA}1/")))
        .and(query_param("expand", "linked_giftcard"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&medium))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{MEDIA}lookup/")))
        .and(body_json(
            json!({"identifier": "ABCDEFGH", "type": "barcode"}),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(&medium))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(MEDIA))
        .and(body_json(json!({
            "identifier": "ABCDEFGH",
            "type": "barcode",
            "active": true,
            "linked_orderpositions": [],
            "info": {},
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&medium))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{MEDIA}1/")))
        .and(body_json(
            json!({"linked_orderpositions": [13, 29], "notes": null}),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("reusable_media/medium_linked")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let media = client.organizer("bigevents").reusable_media();
    media
        .get_with(1, &Expand::new([ReusableMediumExpand::LinkedGiftcard]))
        .await
        .unwrap();
    let lookup = LookupMedium::builder()
        .identifier("ABCDEFGH")
        .r#type(MediumType::Barcode)
        .build();
    media.lookup(&lookup).await.unwrap();
    let body = CreateReusableMedium::builder()
        .identifier("ABCDEFGH")
        .r#type(MediumType::Barcode)
        .active(true)
        .linked_orderpositions(vec![])
        .info(serde_json::Map::new())
        .build();
    media.create(&body).await.unwrap();
    let update = UpdateReusableMedium::builder()
        .linked_orderpositions(vec![13, 29])
        .notes(None)
        .build();
    let medium = media.update(1, &update).await.unwrap();
    assert_eq!(medium.linked_orderpositions.len(), 2);
}
