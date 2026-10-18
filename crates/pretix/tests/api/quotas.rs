use pretix::query::Ordering;
use pretix::resources::quotas::{
    CreateQuota, GetQuota, ListQuotas, Quota, QuotaAvailability, QuotaOrdering, UpdateQuota,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn quota_models() {
    let quota = assert_model::<Quota>(&fixture("quotas/quota"), &[]);
    assert_eq!(quota.available, None);
    assert_model::<QuotaAvailability>(&fixture("quotas/availability"), &[]);

    let mut with_availability = fixture("quotas/quota");
    with_availability["release_after_exit"] = json!(true);
    with_availability["available"] = json!(true);
    with_availability["available_number"] = json!(null);
    let quota = assert_model::<Quota>(&with_availability, &[]);
    assert!(quota.release_after_exit);
    assert_eq!(quota.available, Some(true));
}

#[test]
fn list_params_roundtrip() {
    let params = ListQuotas::builder()
        .subevent_in(vec![1, 2])
        .items_in(vec![3])
        .build();
    let value = serde_json::to_value(&params).unwrap();
    assert_eq!(value, json!({"subevent__in": "1,2", "items__in": "3"}));
    assert_eq!(serde_json::from_value::<ListQuotas>(value).unwrap(), params);
}

#[tokio::test]
async fn list_with_filters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/quotas/",
        ))
        .and(query_param("ordering", "-size"))
        .and(query_param("subevent", "3"))
        .and(query_param("subevent__in", "3,4"))
        .and(query_param("items__in", "1,2"))
        .and(query_param("with_availability", "true"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("quotas/quota")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListQuotas::builder()
        .ordering(Ordering::desc(QuotaOrdering::Size))
        .subevent(3)
        .subevent_in(vec![3, 4])
        .items_in(vec![1, 2])
        .with_availability(true)
        .build();
    let quotas = client
        .organizer("bigevents")
        .event("sampleconf")
        .quotas()
        .list(&params)
        .all()
        .await
        .unwrap();
    assert_eq!(quotas[0].size, Some(200));
}

#[tokio::test]
async fn get_and_availability() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/quotas/1/",
        ))
        .and(query_param("with_availability", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("quotas/quota")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/quotas/1/availability/",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("quotas/availability")))
        .expect(1)
        .mount(&server)
        .await;

    let quotas = client.organizer("bigevents").event("sampleconf").quotas();
    quotas
        .get_with(1, &GetQuota::builder().with_availability(true).build())
        .await
        .unwrap();
    let availability = quotas.availability(1).await.unwrap();
    assert_eq!(availability.available_number, Some(419));
    assert_eq!(availability.blocking_vouchers, 126);
}

#[tokio::test]
async fn create_update_delete() {
    let (server, client) = setup().await;
    let quota = fixture("quotas/quota");
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/quotas/",
        ))
        .and(body_json(json!({
            "name": "Ticket Quota",
            "size": 200,
            "items": [1, 2],
            "variations": [1, 4, 5, 7],
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&quota))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/quotas/1/",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&quota))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/quotas/1/",
        ))
        .and(body_json(json!({"name": "New Ticket Quota", "size": null})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&quota))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/quotas/1/",
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let quotas = client.organizer("bigevents").event("sampleconf").quotas();
    let body = CreateQuota::builder()
        .name("Ticket Quota")
        .size(200)
        .items(vec![1, 2])
        .variations(vec![1, 4, 5, 7])
        .build();
    quotas.create(&body).await.unwrap();
    quotas.get(1).await.unwrap();
    let update = UpdateQuota::builder()
        .name("New Ticket Quota")
        .size(None)
        .build();
    quotas.update(1, &update).await.unwrap();
    quotas.delete(1).await.unwrap();
}
