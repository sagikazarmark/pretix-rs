use http::StatusCode;
use jiff::Timestamp;
use pretix::Error;
use pretix::query::{Expand, Ordering};
use pretix::resources::checkin::{
    AnnulCheckin, AnnulmentResult, Checkin, CheckinErrorReason, CheckinOrdering, CheckinPosition,
    CheckinPositionExpand, CheckinPositionOrdering, CheckinResult, CheckinStatus, CheckinType,
    ListCheckins, RedeemTicket, SearchTickets,
};
use pretix::resources::items::MediaPolicy;
use pretix::resources::orders::OrderStatus;
use pretix::resources::reusable_media::MediumType;
use serde_json::{Value, json};
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn checkin_models() {
    assert_model::<CheckinResult>(&fixture("checkin/redeem_ok"), &[]);
    assert_model::<CheckinResult>(&fixture("checkin/redeem_incomplete"), &[]);
    assert_model::<CheckinResult>(&fixture("checkin/redeem_invalid"), &[]);
    assert_model::<CheckinResult>(&fixture("checkin/redeem_unpaid"), &[]);
    assert_model::<AnnulmentResult>(&fixture("checkin/annul"), &[]);
    assert_model::<Checkin>(&fixture("checkin/checkin"), &[]);
    let position = assert_model::<CheckinPosition>(&fixture("checkin/position"), &[]);
    assert_eq!(position.order_status, Some(OrderStatus::Paid));
    assert_eq!(position.position.print_logs.len(), 1);
}

#[test]
fn exchange_result() {
    let mut json = fixture("checkin/redeem_ok");
    json["status"] = json!("exchange");
    json["media_policy"] = json!("reuse_or_new");
    json["media_type"] = json!("nfc_mf0aes");
    let result = assert_model::<CheckinResult>(&json, &[]);
    assert_eq!(result.media_policy, Some(MediaPolicy::ReuseOrNew));
    assert_eq!(result.media_type, Some(MediumType::NfcMf0aes));
}

const REDEEM: &str = "/api/v1/organizers/bigevents/checkinrpc/redeem/";

fn redeem_body() -> RedeemTicket {
    RedeemTicket::builder()
        .secret("M5BO19XmFwAjLd4nDYUAL9ISjhti0e9q")
        .lists(vec![1])
        .nonce("Pvrk50vUzQd0DhdpNRL4I4OcXsvg70uA")
        .questions_supported(true)
        .answers([(4, "XS".to_owned())].into())
        .build()
}

async fn redeem_with(status: u16, body: Value) -> pretix::Result<CheckinResult> {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(REDEEM))
        .and(body_json(json!({
            "secret": "M5BO19XmFwAjLd4nDYUAL9ISjhti0e9q",
            "lists": [1],
            "nonce": "Pvrk50vUzQd0DhdpNRL4I4OcXsvg70uA",
            "questions_supported": true,
            "answers": {"4": "XS"},
        })))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;
    client
        .organizer("bigevents")
        .checkin_rpc()
        .redeem(&redeem_body())
        .await
}

#[tokio::test]
async fn redeem_ok() {
    let result = redeem_with(201, fixture("checkin/redeem_ok"))
        .await
        .unwrap();
    assert_eq!(result.status, CheckinStatus::Ok);
    assert_eq!(result.list.unwrap().event, "sampleconf");
    assert_eq!(result.position.unwrap().position.id, 23442);
}

#[tokio::test]
async fn redeem_incomplete_is_not_an_error() {
    let result = redeem_with(400, fixture("checkin/redeem_incomplete"))
        .await
        .unwrap();
    assert_eq!(result.status, CheckinStatus::Incomplete);
    assert_eq!(result.questions.len(), 1);
}

#[tokio::test]
async fn redeem_invalid_ticket_is_not_an_error() {
    let result = redeem_with(404, fixture("checkin/redeem_invalid"))
        .await
        .unwrap();
    assert_eq!(result.status, CheckinStatus::Error);
    assert_eq!(result.reason, Some(CheckinErrorReason::Invalid));
    assert!(result.position.is_none());
}

#[tokio::test]
async fn redeem_known_invalid_ticket() {
    let result = redeem_with(200, fixture("checkin/redeem_unpaid"))
        .await
        .unwrap();
    assert_eq!(result.reason, Some(CheckinErrorReason::Unpaid));
}

#[tokio::test]
async fn redeem_permission_error_is_api_error() {
    let err = redeem_with(
        403,
        json!({"detail": "You do not have permission to perform this action."}),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, Error::Api(_)));
    assert_eq!(err.status(), Some(StatusCode::FORBIDDEN));
}

#[tokio::test]
async fn redeem_validation_error_is_api_error() {
    let err = redeem_with(400, json!({"lists": ["This field is required."]}))
        .await
        .unwrap_err();
    assert_eq!(err.status(), Some(StatusCode::BAD_REQUEST));
    assert!(err.as_api().unwrap().field_errors().is_some());
}

#[tokio::test]
async fn redeem_not_found_without_status_is_api_error() {
    let err = redeem_with(404, json!({"detail": "Not found."}))
        .await
        .unwrap_err();
    assert_eq!(err.status(), Some(StatusCode::NOT_FOUND));
}

#[tokio::test]
async fn redeem_with_expand_and_options() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(REDEEM))
        .and(query_param("expand", "answers.question"))
        .and(body_json(json!({
            "secret": "abc",
            "source_type": "nfc_uid",
            "lists": [1, 2],
            "type": "exit",
            "datetime": "2017-12-25T12:45:23Z",
            "force": true,
            "exchange_medium_type": "barcode",
            "exchange_medium_identifier": "XYZ",
            "use_order_locale": true,
            "simulate": true,
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("checkin/redeem_ok")))
        .expect(1)
        .mount(&server)
        .await;

    let body = RedeemTicket::builder()
        .secret("abc")
        .source_type(MediumType::NfcUid)
        .lists(vec![1, 2])
        .r#type(CheckinType::Exit)
        .datetime("2017-12-25T12:45:23Z".parse::<Timestamp>().unwrap())
        .force(true)
        .exchange_medium_type(MediumType::Barcode)
        .exchange_medium_identifier("XYZ")
        .use_order_locale(true)
        .simulate(true)
        .build();
    client
        .organizer("bigevents")
        .checkin_rpc()
        .redeem_with(
            &body,
            &Expand::new([CheckinPositionExpand::AnswersQuestion]),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn search() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/checkinrpc/search/"))
        .and(query_param("list", "1"))
        .and(query_param("search", "Peter"))
        .and(query_param("ordering", "-last_checked_in"))
        .and(query_param("item__in", "1,2"))
        .and(query_param("order__status", "p"))
        .and(query_param("expand", "item"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("checkin/position")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = SearchTickets::builder()
        .list(vec![1])
        .search("Peter")
        .ordering(Ordering::desc(CheckinPositionOrdering::LastCheckedIn))
        .item_in(vec![1, 2])
        .order_status(OrderStatus::Paid)
        .build();
    let page = client
        .organizer("bigevents")
        .checkin_rpc()
        .search_with(&params, &Expand::new([CheckinPositionExpand::Item]))
        .await
        .unwrap();
    assert_eq!(page.results[0]["attendee_name"], "Peter");
}

#[tokio::test]
async fn search_typed() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/checkinrpc/search/"))
        .and(query_param("list", "1"))
        .and(query_param("secret", "z3fsn8jyufm5kpk768q69gkbyr5f4h6w"))
        .and(query_param("voucher", "17"))
        .and(query_param("ignore_status", "true"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("checkin/position")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = SearchTickets::builder()
        .list(vec![1])
        .secret("z3fsn8jyufm5kpk768q69gkbyr5f4h6w")
        .voucher(17)
        .ignore_status(true)
        .build();
    let page = client
        .organizer("bigevents")
        .checkin_rpc()
        .search(&params)
        .await
        .unwrap();
    let position = &page.results[0];
    assert_eq!(position.position.id, 23442);
    assert_eq!(position.order_status, Some(OrderStatus::Paid));
    assert_eq!(position.require_attention, Some(false));
}

#[tokio::test]
async fn annul() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/bigevents/checkinrpc/annul/"))
        .and(body_json(json!({
            "lists": [1],
            "nonce": "Pvrk50vUzQd0DhdpNRL4I4OcXsvg70uA",
            "error_explanation": "Turnstile did not turn",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("checkin/annul")))
        .expect(1)
        .mount(&server)
        .await;

    let body = AnnulCheckin::builder()
        .lists(vec![1])
        .nonce("Pvrk50vUzQd0DhdpNRL4I4OcXsvg70uA")
        .error_explanation("Turnstile did not turn")
        .build();
    let result = client
        .organizer("bigevents")
        .checkin_rpc()
        .annul(&body)
        .await
        .unwrap();
    assert_eq!(result.status, CheckinStatus::Ok);
}

#[tokio::test]
async fn list_history() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/checkins/",
        ))
        .and(query_param("successful", "false"))
        .and(query_param("error_reason", "already_redeemed"))
        .and(query_param("type", "entry"))
        .and(query_param("created_since", "2017-12-01T00:00:00Z"))
        .and(query_param("ordering", "-datetime"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("checkin/checkin")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListCheckins::builder()
        .successful(false)
        .error_reason(CheckinErrorReason::AlreadyRedeemed)
        .r#type(CheckinType::Entry)
        .created_since("2017-12-01T00:00:00Z".parse::<Timestamp>().unwrap())
        .ordering(Ordering::desc(CheckinOrdering::Datetime))
        .build();
    let page = client
        .organizer("bigevents")
        .event("sampleconf")
        .checkins()
        .list(&params)
        .await
        .unwrap();
    assert_eq!(page.results[0].position, Some(1234));
    assert_eq!(page.results[0].r#type, CheckinType::Entry);
}
