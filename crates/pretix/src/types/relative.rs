use std::fmt;

use jiff::Timestamp;
use jiff::civil::Date;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

const RELDATE_PREFIX: &str = "RELDATE/";

/// A point in time that is either absolute or relative to an event date.
///
/// Relative values look like `RELDATE/3/12:00:00/presale_start/` (3 days before the
/// presale start, at 12:00) and are kept verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelativeDateTime {
    /// An absolute timestamp.
    Absolute(Timestamp),
    /// A relative specification (`RELDATE/…`).
    Relative(String),
}

/// A date that is either absolute or relative to an event date.
///
/// Relative values look like `RELDATE/3/-/presale_start/` and are kept verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelativeDate {
    /// An absolute date.
    Absolute(Date),
    /// A relative specification (`RELDATE/…`).
    Relative(String),
}

impl fmt::Display for RelativeDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Absolute(value) => value.fmt(f),
            Self::Relative(value) => f.write_str(value),
        }
    }
}

impl fmt::Display for RelativeDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Absolute(value) => value.fmt(f),
            Self::Relative(value) => f.write_str(value),
        }
    }
}

impl From<Timestamp> for RelativeDateTime {
    fn from(value: Timestamp) -> Self {
        Self::Absolute(value)
    }
}

impl From<Date> for RelativeDate {
    fn from(value: Date) -> Self {
        Self::Absolute(value)
    }
}

impl Serialize for RelativeDateTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl Serialize for RelativeDate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for RelativeDateTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        if raw.starts_with(RELDATE_PREFIX) {
            return Ok(Self::Relative(raw));
        }
        raw.parse().map(Self::Absolute).map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for RelativeDate {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        if raw.starts_with(RELDATE_PREFIX) {
            return Ok(Self::Relative(raw));
        }
        raw.parse().map(Self::Absolute).map_err(de::Error::custom)
    }
}

/// The schema of the `RELDATE/…` form shared by both types.
#[cfg(feature = "schemars")]
fn relative_schema() -> schemars::Schema {
    schemars::json_schema!({
        "description": "A specification relative to an event date (`RELDATE/…`).",
        "type": "string",
        "pattern": format!("^{RELDATE_PREFIX}"),
    })
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for RelativeDateTime {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "RelativeDateTime".into()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        concat!(module_path!(), "::RelativeDateTime").into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "description": "A point in time that is either absolute or relative to an event date.",
            "anyOf": [generator.subschema_for::<Timestamp>(), relative_schema()],
        })
    }
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for RelativeDate {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "RelativeDate".into()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        concat!(module_path!(), "::RelativeDate").into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "description": "A date that is either absolute or relative to an event date.",
            "anyOf": [generator.subschema_for::<Date>(), relative_schema()],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_forms() {
        let abs: RelativeDateTime = serde_json::from_str(r#""2017-12-27T10:00:00+02:00""#).unwrap();
        assert!(matches!(abs, RelativeDateTime::Absolute(_)));
        let rel: RelativeDateTime =
            serde_json::from_str(r#""RELDATE/3/12:00:00/presale_start/""#).unwrap();
        assert_eq!(
            serde_json::to_string(&rel).unwrap(),
            r#""RELDATE/3/12:00:00/presale_start/""#
        );

        let date: RelativeDate = serde_json::from_str(r#""2017-12-27""#).unwrap();
        assert_eq!(serde_json::to_string(&date).unwrap(), r#""2017-12-27""#);
    }
}
