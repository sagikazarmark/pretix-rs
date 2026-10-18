use jiff::Timestamp;
use pretix::query::{AttrFilters, Ordering};
use pretix::resources::subevents::{
    CreateSubEvent, ListOrganizerSubEvents, ListSubEvents, SubEvent, SubEventItemPriceOverride,
    SubEventOrdering, UpdateSubEvent,
};
use pretix::types::LocalizedString;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn subevent_models() {
    assert_model::<SubEvent>(&fixture("subevents/subevent"), &[]);
    let created = assert_model::<SubEvent>(&fixture("subevents/subevent_created"), &[]);
    assert_eq!(created.event, None);
}

#[tokio::test]
async fn list_with_filters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/subevents/",
        ))
        .and(query_param("ordering", "-last_modified"))
        .and(query_param("active", "true"))
        .and(query_param("modified_since", "2017-12-01T00:00:00Z"))
        .and(query_param("with_availability_for", "web"))
        .and(query_param("attr[Format]", "Seminar"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("subevents/subevent")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListSubEvents::builder()
        .ordering(Ordering::desc(SubEventOrdering::LastModified))
        .active(true)
        .modified_since("2017-12-01T00:00:00Z".parse::<Timestamp>().unwrap())
        .with_availability_for("web")
        .attr(AttrFilters::default().with("Format", "Seminar"))
        .build();
    let page = client
        .organizer("bigevents")
        .event("sampleconf")
        .subevents()
        .list(&params)
        .await
        .unwrap();
    assert_eq!(page.results[0].item_price_overrides[0].item, 2);
}

#[tokio::test]
async fn organizer_list() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/subevents/"))
        .and(query_param("event__live", "true"))
        .and(query_param("sales_channel", "web"))
        .and(query_param("ordering", "date_from"))
        .and(query_param("modified_since", "2017-12-01T00:00:00Z"))
        .and(query_param("with_availability_for", "web"))
        .and(query_param("attr[Format]", "Seminar"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("subevents/subevent")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListOrganizerSubEvents::builder()
        .event_live(true)
        .sales_channel("web")
        .ordering(Ordering::asc(SubEventOrdering::DateFrom))
        .modified_since("2017-12-01T00:00:00Z".parse::<Timestamp>().unwrap())
        .with_availability_for("web")
        .attr(AttrFilters::default().with("Format", "Seminar"))
        .build();
    let subevents = client
        .organizer("bigevents")
        .subevents()
        .list(&params)
        .all()
        .await
        .unwrap();
    assert_eq!(subevents[0].event.as_deref(), Some("sampleconf"));
}

#[tokio::test]
async fn get_create_update_delete() {
    let (server, client) = setup().await;
    let base = "/api/v1/organizers/bigevents/events/sampleconf/subevents/";
    Mock::given(method("GET"))
        .and(path(format!("{base}1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("subevents/subevent")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(base))
        .and(body_json(json!({
            "name": {"en": "First Sample Conference"},
            "date_from": "2017-12-27T10:00:00Z",
            "active": false,
            "item_price_overrides": [{
                "item": 2,
                "disabled": false,
                "available_from": null,
                "available_until": null,
                "price": "12.00",
            }],
            "meta_data": {"Format": "Seminar"},
            "comment": "Internal note",
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(fixture("subevents/subevent_created")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{base}1/")))
        .and(body_json(json!({
            "name": {"en": "New Subevent Name"},
            "date_to": null,
            "comment": null,
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("subevents/subevent")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{base}1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let subevents = client
        .organizer("bigevents")
        .event("sampleconf")
        .subevents();
    let subevent = subevents.get(1).await.unwrap();
    assert_eq!(subevent.id, 1);

    let body = CreateSubEvent::builder()
        .name(LocalizedString::from([("en", "First Sample Conference")]))
        .date_from("2017-12-27T10:00:00Z".parse().unwrap())
        .active(false)
        .item_price_overrides(vec![
            SubEventItemPriceOverride::builder()
                .item(2)
                .price("12.00".parse().unwrap())
                .build(),
        ])
        .meta_data([("Format".to_owned(), "Seminar".to_owned())].into())
        .comment("Internal note")
        .build();
    subevents.create(&body).await.unwrap();

    let update = UpdateSubEvent::builder()
        .name(LocalizedString::from([("en", "New Subevent Name")]))
        .date_to(None)
        .comment(None)
        .build();
    subevents.update(1, &update).await.unwrap();
    subevents.delete(1).await.unwrap();
}

#[tokio::test]
async fn create_without_meta_data() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/subevents/",
        ))
        .and(body_json(json!({
            "name": {"en": "First Sample Conference"},
            "date_from": "2017-12-27T10:00:00Z",
            "meta_data": {},
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(fixture("subevents/subevent_created")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = CreateSubEvent::builder()
        .name(LocalizedString::from([("en", "First Sample Conference")]))
        .date_from("2017-12-27T10:00:00Z".parse().unwrap())
        .build();
    client
        .organizer("bigevents")
        .event("sampleconf")
        .subevents()
        .create(&body)
        .await
        .unwrap();
}
