use pretix::ListParams;
use pretix::resources::imported_secrets::{
    CreateImportedSecret, ImportedSecret, UpdateImportedSecret,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const SECRETS: &str = "/api/v1/organizers/bigevents/events/sampleconf/imported_secrets/";

#[test]
fn imported_secret_model() {
    assert_model::<ImportedSecret>(&fixture("imported_secrets/imported_secret"), &[]);
}

#[tokio::test]
async fn list_and_get() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(SECRETS))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("imported_secrets/imported_secret")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{SECRETS}1/")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("imported_secrets/imported_secret")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let secrets = client
        .organizer("bigevents")
        .event("sampleconf")
        .imported_secrets();
    let page = secrets.list(&ListParams::default()).await.unwrap();
    assert_eq!(page.results[0].secret, "foobar");
    assert!(!secrets.get(1).await.unwrap().used);
}

#[tokio::test]
async fn create_bulk_create_update_delete() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(SECRETS))
        .and(body_json(json!({"secret": "foobar"})))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(fixture("imported_secrets/imported_secret")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{SECRETS}bulk_create/")))
        .and(body_json(json!([
            {"secret": "foobar", "used": false},
            {"secret": "baz", "item": 2},
        ])))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 1, "secret": "foobar", "used": false, "item": null, "variation": null, "subevent": null},
            {"id": 2, "secret": "baz", "used": false, "item": 2, "variation": null, "subevent": null},
        ])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{SECRETS}1/")))
        .and(body_json(json!({"item": 2, "subevent": null})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("imported_secrets/imported_secret")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{SECRETS}1/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let secrets = client
        .organizer("bigevents")
        .event("sampleconf")
        .imported_secrets();
    secrets
        .create(&CreateImportedSecret::builder().secret("foobar").build())
        .await
        .unwrap();
    let bulk = [
        CreateImportedSecret::builder()
            .secret("foobar")
            .used(false)
            .build(),
        CreateImportedSecret::builder()
            .secret("baz")
            .item(2)
            .build(),
    ];
    let created = secrets.bulk_create(&bulk).await.unwrap();
    assert_eq!(created.len(), 2);
    assert_eq!(created[1].item, Some(2));
    let update = UpdateImportedSecret::builder()
        .item(Some(2))
        .subevent(None)
        .build();
    secrets.update(1, &update).await.unwrap();
    secrets.delete(1).await.unwrap();
}
