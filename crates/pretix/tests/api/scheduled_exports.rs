use pretix::query::Ordering;
use pretix::resources::scheduled_exports::{
    CreateScheduledExport, ListScheduledExports, ScheduledExport, ScheduledExportOrdering,
    UpdateScheduledExport,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn scheduled_export_models() {
    let export = assert_model::<ScheduledExport>(&fixture("scheduled_exports/event_export"), &[]);
    assert_eq!(export.timezone, None);
    let export =
        assert_model::<ScheduledExport>(&fixture("scheduled_exports/organizer_export"), &[]);
    assert_eq!(export.timezone.as_deref(), Some("Europe/Berlin"));
}

#[tokio::test]
async fn event_scheduled_exports() {
    let (server, client) = setup().await;
    let export = fixture("scheduled_exports/event_export");
    let base = "/api/v1/organizers/bigevents/events/sampleconf/scheduled_exports/";
    Mock::given(method("GET"))
        .and(path(base))
        .and(query_param("ordering", "-schedule_next_run"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_of(vec![export.clone()])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&export))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(base))
        .and(body_json(json!({
            "export_identifier": "orderlist",
            "export_form_data": {"_format": "xlsx", "date_range": "week_previous"},
            "locale": "en",
            "mail_additional_recipients": "mary@example.org",
            "mail_subject": "Order list",
            "mail_template": "Here is last week's order list\n\nCheers\nJohn",
            "schedule_rrule": "DTSTART:20230118T000000\nRRULE:FREQ=WEEKLY;BYDAY=TU,WE,TH",
            "schedule_rrule_time": "04:00:00",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&export))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{base}1/")))
        .and(body_json(json!({
            "export_form_data": {"_format": "xlsx", "date_range": "week_this"},
            "schedule_rrule": null,
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(&export))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{base}1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let exports = client
        .organizer("bigevents")
        .event("sampleconf")
        .scheduled_exports();
    let params = ListScheduledExports::builder()
        .ordering(Ordering::desc(ScheduledExportOrdering::ScheduleNextRun))
        .build();
    let page = exports.list(&params).await.unwrap();
    assert_eq!(page.results[0].owner, "john@example.com");
    exports.get(1).await.unwrap();

    let form_data = json!({"_format": "xlsx", "date_range": "week_previous"});
    let body = CreateScheduledExport::builder()
        .export_identifier("orderlist")
        .export_form_data(form_data.as_object().unwrap().clone())
        .locale("en")
        .mail_additional_recipients("mary@example.org")
        .mail_subject("Order list")
        .mail_template("Here is last week's order list\n\nCheers\nJohn")
        .schedule_rrule("DTSTART:20230118T000000\nRRULE:FREQ=WEEKLY;BYDAY=TU,WE,TH")
        .schedule_rrule_time("04:00:00".parse().unwrap())
        .build();
    exports.create(&body).await.unwrap();

    let form_data = json!({"_format": "xlsx", "date_range": "week_this"});
    let update = UpdateScheduledExport::builder()
        .export_form_data(form_data.as_object().unwrap().clone())
        .schedule_rrule(None)
        .build();
    exports.update(1, &update).await.unwrap();
    exports.delete(1).await.unwrap();
}

#[tokio::test]
async fn organizer_scheduled_exports() {
    let (server, client) = setup().await;
    let export = fixture("scheduled_exports/organizer_export");
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/scheduled_exports/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_of(vec![export.clone()])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/organizers/bigevents/scheduled_exports/"))
        .and(body_json(json!({
            "export_identifier": "orderlist",
            "locale": "en",
            "mail_subject": "Order list",
            "mail_template": "Hi",
            "schedule_rrule_time": "04:00:00",
            "timezone": "Europe/Berlin",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&export))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/organizers/bigevents/scheduled_exports/1/"))
        .and(body_json(json!({"timezone": "UTC"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&export))
        .expect(1)
        .mount(&server)
        .await;

    let exports = client.organizer("bigevents").scheduled_exports();
    let page = exports
        .list(&ListScheduledExports::default())
        .await
        .unwrap();
    assert_eq!(page.results[0].timezone.as_deref(), Some("Europe/Berlin"));
    let body = CreateScheduledExport::builder()
        .export_identifier("orderlist")
        .locale("en")
        .mail_subject("Order list")
        .mail_template("Hi")
        .schedule_rrule_time("04:00".parse().unwrap())
        .timezone("Europe/Berlin")
        .build();
    exports.create(&body).await.unwrap();
    exports
        .update(1, &UpdateScheduledExport::builder().timezone("UTC").build())
        .await
        .unwrap();
}
