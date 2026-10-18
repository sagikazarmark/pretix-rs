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

/// A response model: deserializable, serializable and comparable, and with the
/// `schemars` feature also described by a JSON Schema.
#[cfg(not(feature = "schemars"))]
pub trait Model: DeserializeOwned + Serialize + PartialEq + Debug {}

#[cfg(not(feature = "schemars"))]
impl<T: DeserializeOwned + Serialize + PartialEq + Debug> Model for T {}

/// A response model: deserializable, serializable and comparable, and with the
/// `schemars` feature also described by a JSON Schema.
#[cfg(feature = "schemars")]
pub trait Model: DeserializeOwned + Serialize + PartialEq + Debug + schemars::JsonSchema {}

#[cfg(feature = "schemars")]
impl<T: DeserializeOwned + Serialize + PartialEq + Debug + schemars::JsonSchema> Model for T {}

/// Checks that `json` (typically a documentation example) is fully modeled by `T`:
///
/// 1. it deserializes into `T`,
/// 2. `T` survives a serialize/deserialize round trip unchanged (required for journaling),
/// 3. every key of the original JSON is present after re-serializing (no field was
///    dropped because of a missing or misspelled model field),
/// 4. with the `schemars` feature, the original JSON matches the deserialization schema
///    of `T` and the re-serialized JSON matches its serialization schema.
///
/// Keys in `ignored` (dotted paths like `positions.downloads`) are excluded from (3).
pub fn assert_model<T: Model>(json: &Value, ignored: &[&str]) -> T {
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

    #[cfg(feature = "schemars")]
    {
        use schemars::generate::SchemaSettings;

        assert_schema::<T>(SchemaSettings::default().for_deserialize(), json);
        assert_schema::<T>(SchemaSettings::default().for_serialize(), &reserialized);
    }

    model
}

/// Validates `instance` against the schema of `T` generated with `settings`.
#[cfg(feature = "schemars")]
#[track_caller]
pub fn assert_schema<T: schemars::JsonSchema>(
    settings: schemars::generate::SchemaSettings,
    instance: &Value,
) {
    let schema = settings.into_generator().into_root_schema_for::<T>();
    let validator = jsonschema::options()
        .should_validate_formats(true)
        .build(schema.as_value())
        .unwrap_or_else(|err| panic!("invalid schema for {}: {err}", T::schema_name()));
    let errors: Vec<String> = validator
        .iter_errors(instance)
        .map(|err| format!("{}: {err}", err.instance_path()))
        .collect();
    assert!(
        errors.is_empty(),
        "JSON does not match the schema of {}: {errors:#?}",
        T::schema_name()
    );
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
