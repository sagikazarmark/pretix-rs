use pretix::query::Expand;
use pretix::resources::seats::{
    BulkBlockSeats, BulkUnblockSeats, ListSeats, Seat, SeatExpansion, UpdateSeat,
};
use pretix::types::Expandable;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn seat_models() {
    assert_model::<Seat>(&fixture("seats/seat"), &[]);
    let seat = assert_model::<Seat>(&fixture("seats/seat_in_list"), &[]);
    assert_eq!(seat.voucher, Some(Expandable::Id(51)));
    let seat = assert_model::<Seat>(&fixture("seats/seat_expanded"), &[]);
    let position = seat.orderposition.unwrap().into_expanded().unwrap();
    assert_eq!(position["order"]["code"], "U0HW7");
}

#[tokio::test]
async fn list_event_seats() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/seats/",
        ))
        .and(query_param("zone_name", "Ground floor"))
        .and(query_param("row_name", "1"))
        .and(query_param("row_label", "A"))
        .and(query_param("seat_number", "2"))
        .and(query_param("seat_label", "B"))
        .and(query_param("seat_guid", "abc"))
        .and(query_param("blocked", "false"))
        .and(query_param("is_available", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_of(vec![
            fixture("seats/seat_in_list"),
            fixture("seats/seat"),
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let params = ListSeats::builder()
        .zone_name("Ground floor")
        .row_name("1")
        .row_label("A")
        .seat_number("2")
        .seat_label("B")
        .seat_guid("abc")
        .blocked(false)
        .is_available(true)
        .build();
    let page = client
        .organizer("bigevents")
        .event("sampleconf")
        .seats()
        .list(&params)
        .await
        .unwrap();
    assert_eq!(page.results.len(), 2);
}

#[tokio::test]
async fn list_subevent_seats_with_expand() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/subevents/7/seats/",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("seats/seat_expanded")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListSeats::builder()
        .expand(vec![SeatExpansion::OrderPosition, SeatExpansion::Voucher])
        .build();
    let request = client
        .organizer("bigevents")
        .event("sampleconf")
        .subevents()
        .seats(7)
        .list(&params);
    assert_eq!(
        request.url().query(),
        Some("expand=orderposition&expand=voucher")
    );
    request.await.unwrap();
}

#[tokio::test]
async fn get_and_update() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/seats/1634/",
        ))
        .and(query_param("expand", "orderposition"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("seats/seat_expanded")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/seats/1636/",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("seats/seat")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/subevents/7/seats/1636/",
        ))
        .and(body_json(json!({"blocked": true})))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("seats/seat")))
        .expect(1)
        .mount(&server)
        .await;

    let event = client.organizer("bigevents").event("sampleconf");
    let seat = event
        .seats()
        .get_with(1634, &Expand::new([SeatExpansion::OrderPosition]))
        .await
        .unwrap();
    assert!(seat.orderposition.is_some());
    event.seats().get(1636).await.unwrap();
    let seat = event
        .subevents()
        .seats(7)
        .update(1636, &UpdateSeat::builder().blocked(true).build())
        .await
        .unwrap();
    assert!(seat.blocked);
}

#[tokio::test]
async fn bulk_block_and_unblock() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/seats/bulk_block/",
        ))
        .and(body_json(json!({"ids": [12, 45, 56]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/subevents/7/seats/bulk_unblock/",
        ))
        .and(body_json(json!({
            "seat_guids": ["6c0e29e5-05d6-421f-99f3-afd01478ecad"],
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let event = client.organizer("bigevents").event("sampleconf");
    event
        .seats()
        .bulk_block(&BulkBlockSeats::builder().ids(vec![12, 45, 56]).build())
        .await
        .unwrap();
    let body = BulkUnblockSeats::builder()
        .seat_guids(vec!["6c0e29e5-05d6-421f-99f3-afd01478ecad".into()])
        .build();
    event
        .subevents()
        .seats(7)
        .bulk_unblock(&body)
        .await
        .unwrap();
}
