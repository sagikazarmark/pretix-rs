use pretix::resources::customers::{CreateCustomer, Customer, ListCustomers, UpdateCustomer};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const CUSTOMERS: &str = "/api/v1/organizers/bigevents/customers/";

#[test]
fn customer_models() {
    assert_model::<Customer>(&fixture("customers/customer"), &[]);
    let anonymized = assert_model::<Customer>(&fixture("customers/customer_anonymized"), &[]);
    assert!(anonymized.email.is_none());
}

#[tokio::test]
async fn list_and_get() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(CUSTOMERS))
        .and(query_param("email", "customer@example.org"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("customers/customer")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{CUSTOMERS}8WSAJCJ/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("customers/customer")))
        .expect(1)
        .mount(&server)
        .await;

    let customers = client.organizer("bigevents").customers();
    let params = ListCustomers::builder()
        .email("customer@example.org")
        .build();
    let page = customers.list(&params).await.unwrap();
    assert_eq!(page.results[0].name_parts["full_name"], "John Doe");
    customers.get("8WSAJCJ").await.unwrap();
}

#[tokio::test]
async fn create_update_anonymize() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(CUSTOMERS))
        .and(body_json(json!({
            "email": "test@example.org",
            "phone": "+493012345678",
            "password": "verysecret",
            "send_email": true,
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("customers/customer")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{CUSTOMERS}8WSAJCJ/")))
        .and(body_json(
            json!({"email": "test@example.org", "notes": null}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("customers/customer")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{CUSTOMERS}8WSAJCJ/anonymize/")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("customers/customer_anonymized")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let customers = client.organizer("bigevents").customers();
    let body = CreateCustomer::builder()
        .email("test@example.org")
        .phone("+493012345678")
        .password("verysecret")
        .send_email(true)
        .build();
    customers.create(&body).await.unwrap();
    let update = UpdateCustomer::builder()
        .email("test@example.org")
        .notes(None)
        .build();
    customers.update("8WSAJCJ", &update).await.unwrap();
    let customer = customers.anonymize("8WSAJCJ").await.unwrap();
    assert!(customer.phone.is_none());
}
