use pretix::ListParams;
use pretix::resources::seating_plans::{CreateSeatingPlan, SeatingPlan, UpdateSeatingPlan};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn seating_plan_model() {
    let plan = assert_model::<SeatingPlan>(&fixture("seating_plans/seating_plan"), &[]);
    assert_eq!(plan.layout["size"]["width"], 800);
}

#[tokio::test]
async fn list_and_get() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/seatingplans/"))
        .and(query_param("page", "2"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("seating_plans/seating_plan")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/seatingplans/2/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("seating_plans/seating_plan")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let plans = client.organizer("bigevents").seating_plans();
    let page = plans
        .list(&ListParams::builder().page(2).build())
        .await
        .unwrap();
    assert_eq!(page.results[0].name, "Main plan");
    plans.get(2).await.unwrap();
}

#[tokio::test]
async fn create_update_delete() {
    let (server, client) = setup().await;
    let plan = fixture("seating_plans/seating_plan");
    let layout = plan["layout"].clone();
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/bigevents/seatingplans/"))
        .and(body_json(json!({"name": "Main plan", "layout": layout})))
        .respond_with(ResponseTemplate::new(201).set_body_json(&plan))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/organizers/bigevents/seatingplans/1/"))
        .and(body_json(json!({"name": "Old plan"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&plan))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/organizers/bigevents/seatingplans/1/"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let plans = client.organizer("bigevents").seating_plans();
    let body = CreateSeatingPlan::builder()
        .name("Main plan")
        .layout(layout)
        .build();
    plans.create(&body).await.unwrap();
    plans
        .update(1, &UpdateSeatingPlan::builder().name("Old plan").build())
        .await
        .unwrap();
    plans.delete(1).await.unwrap();
}
