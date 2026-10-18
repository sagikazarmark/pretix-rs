use std::str::FromStr;

use pretix::ListParams;
use pretix::query::Expand;
use pretix::resources::gift_cards::{
    CreateGiftCard, GiftCard, GiftCardExpand, GiftCardTransaction, IncludeAccepted, ListGiftCards,
    TicketRef, TransactGiftCard, UpdateGiftCard,
};
use pretix::types::Decimal;
use pretix::types::Expandable;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const CARDS: &str = "/api/v1/organizers/bigevents/giftcards/";

fn dec(value: &str) -> Decimal {
    Decimal::from_str(value).unwrap()
}

#[test]
fn gift_card_models() {
    assert_model::<GiftCard>(&fixture("gift_cards/gift_card"), &[]);
    assert_model::<GiftCardTransaction>(&fixture("gift_cards/transaction"), &[]);

    let mut json = fixture("gift_cards/gift_card");
    json["owner_ticket"] = json!(23442);
    let card = assert_model::<GiftCard>(&json, &[]);
    assert_eq!(card.owner_ticket, Some(Expandable::Id(23442)));
}

#[tokio::test]
async fn list_and_get() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(CARDS))
        .and(query_param("value", "13.37"))
        .and(query_param("expired", "false"))
        .and(query_param("include_accepted", "true"))
        .and(query_param("expand", "owner_ticket"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("gift_cards/gift_card")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{CARDS}1/")))
        .and(query_param_is_missing("include_accepted"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("gift_cards/gift_card")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{CARDS}1/")))
        .and(query_param("include_accepted", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("gift_cards/gift_card")))
        .expect(1)
        .mount(&server)
        .await;

    let cards = client.organizer("bigevents").gift_cards();
    let params = ListGiftCards::builder()
        .value(dec("13.37"))
        .expired(false)
        .include_accepted(true)
        .expand(vec![GiftCardExpand::OwnerTicket])
        .build();
    let page = cards.list(&params).await.unwrap();
    assert_eq!(page.results[0].value, dec("13.37"));
    cards.get(1).await.unwrap();
    cards
        .get_with(
            1,
            &IncludeAccepted::builder().include_accepted(true).build(),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn create_update_transact() {
    let (server, client) = setup().await;
    let card = fixture("gift_cards/gift_card");
    Mock::given(method("POST"))
        .and(path(CARDS))
        .and(query_param("expand", "owner_ticket"))
        .and(body_json(json!({
            "secret": "HLBYVELFRC77NCQY",
            "currency": "EUR",
            "value": "13.37",
            "owner_ticket": "z3fsn8jyufm5kpk768q69gkbyr5f4h6w",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&card))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{CARDS}1/")))
        .and(body_json(json!({"value": "14.00", "expires": null})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&card))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{CARDS}1/transact/")))
        .and(query_param_is_missing("include_accepted"))
        .and(body_json(json!({
            "value": "2.00",
            "text": "Optional value explaining the transaction",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(&card))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{CARDS}2/transact/")))
        .and(query_param("include_accepted", "true"))
        .and(body_json(json!({"value": "-100.00"})))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({"value": ["Insufficient"]})))
        .expect(1)
        .mount(&server)
        .await;

    let cards = client.organizer("bigevents").gift_cards();
    let body = CreateGiftCard::builder()
        .secret("HLBYVELFRC77NCQY")
        .currency("EUR")
        .value(dec("13.37"))
        .owner_ticket(TicketRef::Secret("z3fsn8jyufm5kpk768q69gkbyr5f4h6w".into()))
        .build();
    cards
        .create_with(&body, &Expand::new([GiftCardExpand::OwnerTicket]))
        .await
        .unwrap();
    let update = UpdateGiftCard::builder()
        .value(dec("14.00"))
        .expires(None)
        .build();
    cards.update(1, &update).await.unwrap();
    let transact = TransactGiftCard::builder()
        .value(dec("2.00"))
        .text("Optional value explaining the transaction")
        .build();
    cards.transact(1, &transact).await.unwrap();
    let err = cards
        .transact_with(
            2,
            &TransactGiftCard::builder().value(dec("-100.00")).build(),
            &IncludeAccepted::builder().include_accepted(true).build(),
        )
        .await
        .unwrap_err();
    assert_eq!(err.status().map(|s| s.as_u16()), Some(409));
}

#[tokio::test]
async fn transactions() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(format!("{CARDS}1/transactions/")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("gift_cards/transaction")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let page = client
        .organizer("bigevents")
        .gift_cards()
        .transactions(1, &ListParams::default())
        .await
        .unwrap();
    assert_eq!(page.results[0].order.as_deref(), Some("FXQYW"));
}
