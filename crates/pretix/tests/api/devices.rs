use pretix::ListParams;
use pretix::resources::devices::{CreateDevice, Device, UpdateDevice};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const DEVICES: &str = "/api/v1/organizers/bigevents/devices/";

#[test]
fn device_models() {
    assert_model::<Device>(&fixture("devices/device"), &[]);
    assert_model::<Device>(&fixture("devices/device_created"), &[]);
}

#[tokio::test]
async fn crud() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(DEVICES))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("devices/device")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{DEVICES}1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("devices/device")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(DEVICES))
        .and(body_json(
            json!({"name": "Scanner", "all_events": true, "limit_events": []}),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("devices/device_created")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{DEVICES}1/")))
        .and(body_json(json!({"name": "Foo"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("devices/device")))
        .expect(1)
        .mount(&server)
        .await;

    let devices = client.organizer("bigevents").devices();
    let page = devices.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].limit_events, ["museum"]);
    assert_eq!(
        devices.get(1).await.unwrap().software_brand.as_deref(),
        Some("pretixSCAN")
    );
    let body = CreateDevice::builder()
        .name("Scanner")
        .all_events(true)
        .limit_events(vec![])
        .build();
    let device = devices.create(&body).await.unwrap();
    assert_eq!(
        device.initialization_token.as_deref(),
        Some("frkso3m2w58zuw70")
    );
    devices
        .update(1, &UpdateDevice::builder().name("Foo").build())
        .await
        .unwrap();
}
