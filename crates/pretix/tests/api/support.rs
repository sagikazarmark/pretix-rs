//! Shared helpers for the API tests.

use std::fmt::Debug;
use std::path::PathBuf;

use pretix::{Client, RetryPolicy};
use pretty_assertions::assert_eq;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use wiremock::MockServer;

/// Starts a mock server and a client pointed at it (retries disabled, token `secret`).
pub async fn setup() -> (MockServer, Client) {
    let server = MockServer::start().await;
    let client = Client::builder()
        .base_url(format!("{}/api/v1/", server.uri()))
        .token("secret")
        .retry_policy(RetryPolicy::disabled())
        .build()
        .unwrap();
    (server, client)
}

/// Loads `tests/fixtures/<name>.json`.
pub fn fixture(name: &str) -> Value {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "fixtures",
        &format!("{name}.json"),
    ]
    .iter()
    .collect();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("cannot read fixture {}: {err}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("fixture {} is not valid JSON: {err}", path.display()))
}

/// Wraps results in a single-page list response.
pub fn page_of(results: Vec<Value>) -> Value {
    let mut page = serde_json::json!({
        "count": results.len(),
        "next": null,
        "previous": null,
    });
    page["results"] = Value::Array(results);
    page
}

/// Checks that `json` (typically a documentation example) is fully modeled by `T`:
///
/// 1. it deserializes into `T`,
/// 2. `T` survives a serialize/deserialize round trip unchanged (required for journaling),
/// 3. every key of the original JSON is present after re-serializing (no field was
///    dropped because of a missing or misspelled model field).
///
/// Keys in `ignored` (dotted paths like `positions.downloads`) are excluded from (3).
pub fn assert_model<T>(json: &Value, ignored: &[&str]) -> T
where
    T: DeserializeOwned + Serialize + PartialEq + Debug,
{
    let model: T = {
        let deserializer = json.clone();
        serde_path_to_error::deserialize(deserializer)
            .unwrap_or_else(|err| panic!("cannot deserialize at `{}`: {}", err.path(), err.inner()))
    };
    let reserialized = serde_json::to_value(&model).expect("model serializes");
    let roundtripped: T = serde_json::from_value(reserialized.clone())
        .unwrap_or_else(|err| panic!("re-serialized model does not deserialize: {err}"));
    assert_eq!(model, roundtripped, "model changed during round trip");

    let mut missing = Vec::new();
    collect_missing(json, &reserialized, "", ignored, &mut missing);
    assert!(
        missing.is_empty(),
        "fields not captured by the model: {missing:#?}"
    );
    model
}

fn collect_missing(
    original: &Value,
    modeled: &Value,
    path: &str,
    ignored: &[&str],
    missing: &mut Vec<String>,
) {
    match (original, modeled) {
        (Value::Object(original), Value::Object(modeled)) => {
            for (key, value) in original {
                let child = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                if ignored.contains(&child.as_str()) {
                    continue;
                }
                match modeled.get(key) {
                    Some(modeled) => collect_missing(value, modeled, &child, ignored, missing),
                    None => missing.push(child),
                }
            }
        }
        (Value::Array(original), Value::Array(modeled)) => {
            for (original, modeled) in original.iter().zip(modeled) {
                collect_missing(original, modeled, path, ignored, missing);
            }
        }
        _ => {}
    }
}
