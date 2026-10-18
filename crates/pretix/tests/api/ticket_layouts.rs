use std::time::Duration;

use pretix::resources::exporters::ExportJob;
use pretix::resources::ticket_layouts::{
    CreateTicketLayout, RenderBatch, RenderBatchPart, TicketLayout, TicketLayoutItem,
    UpdateTicketLayout,
};
use pretix::{ListParams, PollOptions};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const EVENT: &str = "/api/v1/organizers/bigevents/events/sampleconf/";

#[test]
fn ticket_layout_models() {
    let layout = assert_model::<TicketLayout>(&fixture("ticket_layouts/ticket_layout"), &[]);
    assert_eq!(layout.item_assignments[0].sales_channel, "web");
    assert_model::<TicketLayoutItem>(&fixture("ticket_layouts/ticket_layout_item"), &[]);
    assert_model::<ExportJob>(&fixture("ticket_layouts/render_job"), &[]);
}

#[tokio::test]
async fn layout_crud() {
    let (server, client) = setup().await;
    let layout = fixture("ticket_layouts/ticket_layout");
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}ticketlayouts/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_of(vec![layout.clone()])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}ticketlayouts/1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&layout))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{EVENT}ticketlayouts/")))
        .and(body_json(json!({
            "name": "Default layout",
            "default": true,
            "layout": [],
            "background": "file:1cd99455",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&layout))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{EVENT}ticketlayouts/1/")))
        .and(body_json(
            json!({"name": "Default layout", "background": null}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&layout))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{EVENT}ticketlayouts/1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}ticketlayoutitems/")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("ticket_layouts/ticket_layout_item")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let event = client.organizer("bigevents").event("sampleconf");
    let layouts = event.ticket_layouts();
    let page = layouts.list(&ListParams::default()).await.unwrap();
    assert!(page.results[0].default);
    layouts.get(1).await.unwrap();
    let body = CreateTicketLayout::builder()
        .name("Default layout")
        .default(true)
        .layout(json!([]))
        .background(pretix::types::FileId::new("file:1cd99455"))
        .build();
    layouts.create(&body).await.unwrap();
    let update = UpdateTicketLayout::builder()
        .name("Default layout")
        .background(None)
        .build();
    layouts.update(1, &update).await.unwrap();
    layouts.delete(1).await.unwrap();

    let items = event
        .ticket_layout_items()
        .list(&ListParams::default())
        .await
        .unwrap();
    assert_eq!(items.results[0].layout, 2);
}

#[tokio::test]
async fn render_batch() {
    let (server, client) = setup().await;
    let download_path = format!("{EVENT}ticketpdfrenderer/download/a/b/");
    Mock::given(method("POST"))
        .and(path(format!("{EVENT}ticketpdfrenderer/render_batch/")))
        .and(body_json(json!({
            "parts": [
                {"orderposition": 55412},
                {"orderposition": 55412, "override_channel": "web"},
                {"orderposition": 55412, "override_layout": 56},
            ],
        })))
        .respond_with(
            ResponseTemplate::new(202)
                .set_body_json(json!({"download": format!("https://pretix.eu{download_path}")})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(download_path.as_str()))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "application/pdf")
                .set_body_bytes("%PDF"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let body = RenderBatch::builder()
        .parts(vec![
            RenderBatchPart::builder().orderposition(55412).build(),
            RenderBatchPart::builder()
                .orderposition(55412)
                .override_channel("web")
                .build(),
            RenderBatchPart::builder()
                .orderposition(55412)
                .override_layout(56)
                .build(),
        ])
        .build();
    let job = client
        .organizer("bigevents")
        .event("sampleconf")
        .ticket_pdf_renderer()
        .render_batch(&body)
        .await
        .unwrap();
    let pdf = job
        .download_job(&client)
        .wait(PollOptions::default().with_interval(Duration::from_millis(1)))
        .await
        .unwrap();
    assert_eq!(pdf.content_type.as_deref(), Some("application/pdf"));
}
