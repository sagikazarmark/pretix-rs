use jiff::Timestamp;
use pretix::query::Ordering;
use pretix::resources::orders::FeeType;
use pretix::resources::transactions::{ListTransactions, Transaction, TransactionOrdering};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn transaction_models() {
    let transaction = assert_model::<Transaction>(&fixture("transactions/transaction"), &[]);
    assert_eq!(transaction.event, None);
    let transaction =
        assert_model::<Transaction>(&fixture("transactions/transaction_organizer"), &[]);
    assert_eq!(transaction.event.as_deref(), Some("sampleconf"));
}

#[tokio::test]
async fn event_list_with_filters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/transactions/",
        ))
        .and(query_param("order", "FOO"))
        .and(query_param("datetime_since", "2017-12-01T00:00:00Z"))
        .and(query_param("tax_rate__in", "0.00,19.00"))
        .and(query_param("fee_type__in", "payment,other"))
        .and(query_param("item__in", "1,2"))
        .and(query_param("ordering", "-datetime"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("transactions/transaction")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListTransactions::builder()
        .order("FOO")
        .datetime_since("2017-12-01T00:00:00Z".parse::<Timestamp>().unwrap())
        .tax_rate_in(vec!["0.00".parse().unwrap(), "19.00".parse().unwrap()])
        .fee_type_in(vec![FeeType::Payment, FeeType::Other])
        .item_in(vec![1, 2])
        .ordering(Ordering::desc(TransactionOrdering::Datetime))
        .build();
    let page = client
        .organizer("bigevents")
        .event("sampleconf")
        .transactions()
        .list(&params)
        .await
        .unwrap();
    assert_eq!(page.results[0].count, 1);
}

#[tokio::test]
async fn organizer_list() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/transactions/"))
        .and(query_param("event", "sampleconf"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("transactions/transaction_organizer")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let page = client
        .organizer("bigevents")
        .transactions()
        .list(&ListTransactions::builder().event("sampleconf").build())
        .await
        .unwrap();
    assert_eq!(page.results[0].event.as_deref(), Some("sampleconf"));
}

#[test]
fn comma_separated_params_roundtrip() {
    let params = ListTransactions::builder()
        .fee_type_in(vec![FeeType::Payment, FeeType::Other])
        .subevent_in(vec![3, 4])
        .build();
    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["fee_type__in"], "payment,other");
    let back: ListTransactions = serde_json::from_value(json).unwrap();
    assert_eq!(back, params);
}
