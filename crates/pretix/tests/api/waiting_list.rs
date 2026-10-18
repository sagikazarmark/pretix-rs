use pretix::query::Ordering;
use pretix::resources::waiting_list::{
    CreateWaitingListEntry, ListWaitingListEntries, UpdateWaitingListEntry, WaitingListEntry,
    WaitingListEntryOrdering,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn entry_models() {
    let entry = assert_model::<WaitingListEntry>(&fixture("waiting_list/entry"), &[]);
    assert_eq!(entry.name, None);

    let mut full = fixture("waiting_list/entry");
    full["name"] = json!("Jane Doe");
    full["name_parts"] = json!({"_scheme": "given_family", "given_name": "Jane"});
    full["phone"] = json!("+49123456789");
    full["priority"] = json!(3);
    let entry = assert_model::<WaitingListEntry>(&full, &[]);
    assert_eq!(entry.phone.as_deref(), Some("+49123456789"));
    assert_eq!(entry.priority, 3);
}

#[tokio::test]
async fn list_with_filters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/waitinglistentries/",
        ))
        .and(query_param("ordering", "-created"))
        .and(query_param("email", "waiting@example.org"))
        .and(query_param("locale", "en"))
        .and(query_param("has_voucher", "false"))
        .and(query_param("item", "2"))
        .and(query_param("variation", "3"))
        .and(query_param("subevent", "4"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("waiting_list/entry")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListWaitingListEntries::builder()
        .ordering(Ordering::desc(WaitingListEntryOrdering::Created))
        .email("waiting@example.org")
        .locale("en")
        .has_voucher(false)
        .item(2)
        .variation(3)
        .subevent(4)
        .build();
    let page = client
        .organizer("bigevents")
        .event("sampleconf")
        .waiting_list_entries()
        .list(&params)
        .await
        .unwrap();
    assert_eq!(page.results[0].email, "waiting@example.org");
}

#[tokio::test]
async fn create_get_update_send_voucher_delete() {
    let (server, client) = setup().await;
    let entry = fixture("waiting_list/entry");
    let base = "/api/v1/organizers/bigevents/events/sampleconf/waitinglistentries/";
    Mock::given(method("POST"))
        .and(path(base))
        .and(body_json(json!({
            "email": "waiting@example.org",
            "item": 3,
            "locale": "de",
            "priority": 5,
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&entry))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&entry))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{base}1/")))
        .and(body_json(
            json!({"item": 4, "variation": null, "priority": -1}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&entry))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{base}1/send_voucher/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{base}1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let waiting_list = client
        .organizer("bigevents")
        .event("sampleconf")
        .waiting_list_entries();
    let body = CreateWaitingListEntry::builder()
        .email("waiting@example.org")
        .item(3)
        .locale("de")
        .priority(5)
        .build();
    waiting_list.create(&body).await.unwrap();
    waiting_list.get(1).await.unwrap();
    let update = UpdateWaitingListEntry::builder()
        .item(4)
        .variation(None)
        .priority(-1)
        .build();
    waiting_list.update(1, &update).await.unwrap();
    waiting_list.send_voucher(1).await.unwrap();
    waiting_list.delete(1).await.unwrap();
}
