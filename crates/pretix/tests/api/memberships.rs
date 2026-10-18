use jiff::Timestamp;
use pretix::resources::memberships::{
    CreateMembership, ListMemberships, Membership, UpdateMembership,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const MEMBERSHIPS: &str = "/api/v1/organizers/bigevents/memberships/";

#[test]
fn membership_model() {
    assert_model::<Membership>(&fixture("memberships/membership"), &[]);
}

#[tokio::test]
async fn crud() {
    let (server, client) = setup().await;
    let membership = fixture("memberships/membership");
    Mock::given(method("GET"))
        .and(path(MEMBERSHIPS))
        .and(query_param("customer", "EGR9SYT"))
        .and(query_param("membership_type", "1"))
        .and(query_param("testmode", "false"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_of(vec![membership.clone()])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{MEMBERSHIPS}2/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&membership))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(MEMBERSHIPS))
        .and(body_json(json!({
            "membership_type": 2,
            "customer": "EGR9SYT",
            "testmode": false,
            "date_start": "2021-04-18T22:00:00Z",
            "date_end": "2021-04-19T22:00:00Z",
            "attendee_name_parts": {
                "_scheme": "title_given_family",
                "family_name": "Doe",
                "given_name": "John",
                "title": "",
            },
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&membership))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{MEMBERSHIPS}1/")))
        .and(body_json(json!({"membership_type": 3})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&membership))
        .expect(1)
        .mount(&server)
        .await;

    let memberships = client.organizer("bigevents").memberships();
    let params = ListMemberships::builder()
        .customer("EGR9SYT")
        .membership_type(1)
        .testmode(false)
        .build();
    let page = memberships.list(&params).await.unwrap();
    assert_eq!(page.results[0].attendee_name_parts["given_name"], "John");
    let fetched = memberships.get(2).await.unwrap();
    assert_eq!(
        fetched.date_start,
        "2021-04-18T22:00:00Z".parse::<Timestamp>().unwrap()
    );

    let name_parts = json!({
        "_scheme": "title_given_family",
        "family_name": "Doe",
        "given_name": "John",
        "title": "",
    });
    let body = CreateMembership::builder()
        .membership_type(2)
        .customer("EGR9SYT")
        .testmode(false)
        .date_start("2021-04-19T00:00:00+02:00".parse().unwrap())
        .date_end("2021-04-20T00:00:00+02:00".parse().unwrap())
        .attendee_name_parts(name_parts.as_object().unwrap().clone())
        .build();
    memberships.create(&body).await.unwrap();
    memberships
        .update(1, &UpdateMembership::builder().membership_type(3).build())
        .await
        .unwrap();
}
