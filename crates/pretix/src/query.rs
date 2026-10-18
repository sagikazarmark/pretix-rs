//! Types shared by list endpoint query parameters.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Query parameters for list endpoints that only support pagination.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListParams {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
}

/// The `expand` query parameter: replaces ID fields of the response with full objects.
///
/// `E` is a resource specific enum of expandable fields. Used as the parameter of
/// `…_with` endpoint methods (e.g. `get_with(id, &Expand::new([…]))`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Expand<E> {
    /// Fields to expand.
    #[serde(default = "Vec::<E>::new", skip_serializing_if = "Vec::is_empty")]
    pub expand: Vec<E>,
}

impl<E> Expand<E> {
    /// Expands the given fields.
    pub fn new(fields: impl IntoIterator<Item = E>) -> Self {
        Self {
            expand: fields.into_iter().collect(),
        }
    }
}

impl<E> Default for Expand<E> {
    fn default() -> Self {
        Self { expand: Vec::new() }
    }
}

impl<E> FromIterator<E> for Expand<E> {
    fn from_iter<I: IntoIterator<Item = E>>(iter: I) -> Self {
        Self::new(iter)
    }
}

/// Response field selection (`include` / `exclude` query parameters).
///
/// Selecting fields changes the shape of responses, so endpoints that support it return
/// `serde_json::Value` from their `…_with` variants.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct FieldSelection {
    /// Only include these fields (dotted paths for nested fields, e.g. `positions.id`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub include: Vec<String>,
    /// Exclude these fields (applied after `include`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub exclude: Vec<String>,
}

/// Sort order of a list (`ordering` query parameter).
///
/// `F` is a resource specific enum of sortable fields.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ordering<F> {
    /// The field to sort by.
    pub field: F,
    /// Sort in descending order.
    pub descending: bool,
}

impl<F> Ordering<F> {
    /// Sorts ascending by `field`.
    pub fn asc(field: F) -> Self {
        Self {
            field,
            descending: false,
        }
    }

    /// Sorts descending by `field`.
    pub fn desc(field: F) -> Self {
        Self {
            field,
            descending: true,
        }
    }
}

impl<F> From<F> for Ordering<F> {
    fn from(field: F) -> Self {
        Self::asc(field)
    }
}

impl<F: fmt::Display> fmt::Display for Ordering<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.descending {
            f.write_str("-")?;
        }
        self.field.fmt(f)
    }
}

impl<F: fmt::Display> Serialize for Ordering<F> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de, F: FromStr> Deserialize<'de> for Ordering<F> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        let (descending, field) = match raw.strip_prefix('-') {
            Some(field) => (true, field),
            None => (false, raw.as_str()),
        };
        let field = field
            .parse()
            .map_err(|_| de::Error::custom(format!("invalid ordering field `{field}`")))?;
        Ok(Self { field, descending })
    }
}

/// A field name, prefixed with `-` for descending order.
///
/// The schema is the same for every `F`, so it does not depend on `F`.
#[cfg(feature = "schemars")]
impl<F> schemars::JsonSchema for Ordering<F> {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Ordering".into()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        concat!(module_path!(), "::Ordering").into()
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "description": "A field name, prefixed with `-` for descending order.",
            "type": "string",
        })
    }
}

/// Filters on event meta data (`attr[<key>]=<value>` query parameters).
///
/// Use with `#[serde(flatten)]` in list parameters. An empty value matches objects
/// that have no value set for the key.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AttrFilters(pub BTreeMap<String, String>);

impl AttrFilters {
    /// Adds a filter.
    #[must_use]
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.0.insert(key.into(), value.into());
        self
    }

    /// Whether no filter is set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<K: Into<String>, V: Into<String>> FromIterator<(K, V)> for AttrFilters {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl Serialize for AttrFilters {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in &self.0 {
            map.serialize_entry(&format!("attr[{key}]"), value)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for AttrFilters {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct AttrVisitor;

        impl<'de> Visitor<'de> for AttrVisitor {
            type Value = AttrFilters;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a map of attr[<key>] filters")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let mut filters = BTreeMap::new();
                while let Some(key) = access.next_key::<String>()? {
                    match key.strip_prefix("attr[").and_then(|k| k.strip_suffix(']')) {
                        Some(inner) => {
                            filters.insert(inner.to_owned(), access.next_value()?);
                        }
                        None => {
                            access.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AttrFilters(filters))
            }
        }

        deserializer.deserialize_map(AttrVisitor)
    }
}

/// An object of `attr[<key>]` string properties, to be flattened into list parameters.
#[cfg(feature = "schemars")]
impl schemars::JsonSchema for AttrFilters {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> std::borrow::Cow<'static, str> {
        "AttrFilters".into()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        concat!(module_path!(), "::AttrFilters").into()
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "patternProperties": {
                r"^attr\[.*\]$": {
                    "description": "Filter on an event meta data value (`attr[<key>]`).",
                    "type": "string",
                },
            },
        })
    }
}

#[cfg(test)]
#[allow(unreachable_pub)]
mod tests {
    use super::*;
    use crate::types::string_enum;

    string_enum! {
        /// Test fields.
        pub enum Field {
            /// Name.
            Name = "name",
        }
    }

    #[derive(Serialize)]
    struct Params {
        #[serde(skip_serializing_if = "Option::is_none")]
        ordering: Option<Ordering<Field>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        live: Option<bool>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        expand: Vec<String>,
        #[serde(flatten)]
        attr: AttrFilters,
    }

    #[test]
    fn serializes_query() {
        let params = Params {
            ordering: Some(Ordering::desc(Field::Name)),
            live: Some(true),
            expand: vec!["item".into(), "subevent".into()],
            attr: AttrFilters::default().with("Format", "Seminar"),
        };
        assert_eq!(
            serde_html_form::to_string(&params).unwrap(),
            "ordering=-name&live=true&expand=item&expand=subevent&attr%5BFormat%5D=Seminar"
        );
    }

    #[test]
    fn ordering_roundtrips() {
        let ordering: Ordering<Field> = serde_json::from_str(r#""-name""#).unwrap();
        assert_eq!(ordering, Ordering::desc(Field::Name));
        assert_eq!(serde_json::to_string(&ordering).unwrap(), r#""-name""#);
    }
}
