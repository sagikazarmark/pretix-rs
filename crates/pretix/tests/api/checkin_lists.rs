use pretix::query::{Expand, FieldSelection, Ordering};
use pretix::resources::checkin::{
    CheckinErrorReason, CheckinPositionExpand, CheckinResult, CheckinStatus,
};
use pretix::resources::checkin_lists::{
    CheckinList, CheckinListExpand, CheckinListOrdering, CheckinListStatus, CreateCheckinList,
    CreateFailedCheckin, ListCheckinListPositions, ListCheckinLists, RedeemPosition,
    UpdateCheckinList,
};
use pretix::resources::orders::OrderStatus;
use pretix::types::Expandable;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const LISTS: &str = "/api/v1/organizers/bigevents/events/sampleconf/checkinlists/";

#[test]
fn checkin_list_models() {
    let list = assert_model::<CheckinList>(&fixture("checkin_lists/checkin_list"), &[]);
    assert!(list.consider_tickets_used);
    assert!(!list.ignore_in_statistics);
    assert_model::<CheckinList>(&fixture("checkin_lists/checkin_list_created"), &[]);
    assert_model::<CheckinListStatus>(&fixture("checkin_lists/status"), &[]);
    assert_model::<CheckinResult>(&fixture("checkin_lists/redeem_ok"), &[]);
    assert_model::<CheckinResult>(&fixture("checkin_lists/redeem_incomplete"), &[]);
    assert_model::<CheckinResult>(&fixture("checkin_lists/redeem_error"), &[]);
}

#[test]
fn expanded_subevent() {
    let mut json = fixture("checkin_lists/checkin_list");
    json["subevent"] = fixture("subevents/subevent");
    let list = assert_model::<CheckinList>(&json, &[]);
    assert!(matches!(list.subevent, Some(Expandable::Expanded(_))));
}

#[tokio::test]
async fn list_with_filters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(LISTS))
        .and(query_param("subevent_match", "3"))
        .and(query_param("ordering", "name"))
        .and(query_param("expand", "subevent"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("checkin_lists/checkin_list")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListCheckinLists::builder()
        .subevent_match(3)
        .ordering(Ordering::asc(CheckinListOrdering::Name))
        .expand(vec![CheckinListExpand::Subevent])
        .build();
    let page = client
        .organizer("bigevents")
        .event("sampleconf")
        .checkin_lists()
        .list(&params)
        .await
        .unwrap();
    assert_eq!(page.results[0].checkin_count, Some(123));
}

#[tokio::test]
async fn list_with_field_selection() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(LISTS))
        .and(query_param("subevent", "3"))
        .and(|request: &wiremock::Request| {
            let excludes: Vec<_> = request
                .url
                .query_pairs()
                .filter(|(key, _)| key == "exclude")
                .map(|(_, value)| value.into_owned())
                .collect();
            excludes == ["checkin_count", "position_count"]
        })
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![json!({"id": 1, "name": "Default list"})])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let fields = FieldSelection::builder()
        .exclude(vec!["checkin_count".into(), "position_count".into()])
        .build();
    let page = client
        .organizer("bigevents")
        .event("sampleconf")
        .checkin_lists()
        .list_with(&ListCheckinLists::builder().subevent(3).build(), &fields)
        .await
        .unwrap();
    assert_eq!(page.results[0]["name"], "Default list");
    assert!(page.results[0].get("checkin_count").is_none());
}

#[tokio::test]
async fn crud_and_status() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(LISTS))
        .and(body_json(json!({
            "name": "VIP entry",
            "all_products": false,
            "limit_products": [1, 2],
            "allow_multiple_entries": false,
            "allow_entry_after_exit": true,
            "addon_match": false,
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(fixture("checkin_lists/checkin_list_created")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{LISTS}1/")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("checkin_lists/checkin_list")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{LISTS}1/status/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("checkin_lists/status")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{LISTS}2/")))
        .and(body_json(json!({"name": "Backstage", "subevent": null})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("checkin_lists/checkin_list_created")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{LISTS}2/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let lists = client
        .organizer("bigevents")
        .event("sampleconf")
        .checkin_lists();
    let body = CreateCheckinList::builder()
        .name("VIP entry")
        .all_products(false)
        .limit_products(vec![1, 2])
        .allow_multiple_entries(false)
        .allow_entry_after_exit(true)
        .addon_match(false)
        .build();
    assert_eq!(lists.create(&body).await.unwrap().id, 2);
    assert_eq!(lists.get(1).await.unwrap().name, "Default list");
    let status = lists.status(1).await.unwrap();
    assert_eq!(status.items[0].variations[1].value, "Blue");
    let update = UpdateCheckinList::builder()
        .name("Backstage")
        .subevent(None)
        .build();
    lists.update(2, &update).await.unwrap();
    lists.delete(2).await.unwrap();
}

#[tokio::test]
async fn failed_checkins() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(format!("{LISTS}1/failed_checkins/")))
        .and(body_json(json!({
            "raw_barcode": "Pvrk50vUzQd0DhdpNRL4I4OcXsvg70uA",
            "error_reason": "canceled",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let body = CreateFailedCheckin::builder()
        .raw_barcode("Pvrk50vUzQd0DhdpNRL4I4OcXsvg70uA")
        .error_reason(CheckinErrorReason::Canceled)
        .build();
    client
        .organizer("bigevents")
        .event("sampleconf")
        .checkin_lists()
        .failed_checkins(1, &body)
        .await
        .unwrap();
}

#[tokio::test]
async fn positions() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(format!("{LISTS}1/positions/")))
        .and(query_param("has_checkin", "false"))
        .and(query_param("subevent__in", "1,2"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("checkin/position")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{LISTS}1/positions/")))
        .and(query_param("voucher", "17"))
        .and(query_param("order__status", "n"))
        .and(query_param("expand", "item"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("checkin/position")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{LISTS}1/positions/23442/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("checkin/position")))
        .expect(1)
        .mount(&server)
        .await;

    let positions = client
        .organizer("bigevents")
        .event("sampleconf")
        .checkin_lists()
        .positions(1);
    let params = ListCheckinListPositions::builder()
        .has_checkin(false)
        .subevent_in(vec![1, 2])
        .build();
    let page = positions.list(&params).await.unwrap();
    assert_eq!(page.count, 1);
    let params = ListCheckinListPositions::builder()
        .voucher(17)
        .order_status(OrderStatus::Pending)
        .build();
    let page = positions
        .list_with(&params, &Expand::new([CheckinPositionExpand::Item]))
        .await
        .unwrap();
    assert_eq!(page.results[0]["order__status"], "p");
    let position = positions.get(23442).await.unwrap();
    assert_eq!(position.position.secret, "z3fsn8jyufm5kpk768q69gkbyr5f4h6w");
}

#[tokio::test]
async fn redeem_position() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(format!("{LISTS}1/positions/234/redeem/")))
        .and(body_json(json!({
            "force": false,
            "ignore_unpaid": false,
            "nonce": "Pvrk50vUzQd0DhdpNRL4I4OcXsvg70uA",
            "questions_supported": true,
            "canceled_supported": true,
            "answers": {"4": "XS"},
        })))
        .respond_with(
            ResponseTemplate::new(400).set_body_json(fixture("checkin_lists/redeem_incomplete")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{LISTS}1/positions/abc%2Fdef/redeem/")))
        .and(query_param("untrusted_input", "true"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("checkin_lists/redeem_error")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{LISTS}1/positions/999/redeem/")))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({"detail": "Forbidden"})))
        .expect(1)
        .mount(&server)
        .await;

    let positions = client
        .organizer("bigevents")
        .event("sampleconf")
        .checkin_lists()
        .positions(1);
    let body = RedeemPosition::builder()
        .force(false)
        .ignore_unpaid(false)
        .nonce("Pvrk50vUzQd0DhdpNRL4I4OcXsvg70uA")
        .questions_supported(true)
        .canceled_supported(true)
        .answers([(4, "XS".to_owned())].into())
        .build();
    let result = positions.redeem(234, &body).await.unwrap();
    assert_eq!(result.status, CheckinStatus::Incomplete);

    let result = positions
        .redeem_secret("abc/def", &RedeemPosition::default())
        .await
        .unwrap();
    assert_eq!(result.reason, Some(CheckinErrorReason::Unpaid));

    let err = positions
        .redeem(999, &RedeemPosition::default())
        .await
        .unwrap_err();
    assert!(matches!(err, pretix::Error::Api(_)));
}
