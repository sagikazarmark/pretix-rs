//! Serde helpers shared by resource models.

use serde::{Deserialize, Deserializer};

/// Deserializes a present field (including `null`) as `Some(..)`.
///
/// Use with `Option<Option<T>>` fields of PATCH bodies, where `None` means "leave
/// unchanged" and `Some(None)` means "set to null":
///
/// ```ignore
/// #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "crate::serde_util::double_option")]
/// pub category: Option<Option<u64>>,
/// ```
#[expect(
    clippy::option_option,
    reason = "PATCH bodies distinguish an absent field from an explicit null"
)]
pub(crate) fn double_option<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

/// Deserializes `null` as the type's default value.
///
/// Use for collection fields that pretix documents as lists/objects but that may be
/// `null` in practice.
pub(crate) fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Option::<T>::deserialize(deserializer).map(Option::unwrap_or_default)
}

/// Accepts booleans as JSON booleans or as the strings `"true"`/`"false"`.
pub(crate) fn lenient_bool<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolOrString {
        Bool(bool),
        String(String),
    }
    match BoolOrString::deserialize(deserializer)? {
        BoolOrString::Bool(value) => Ok(value),
        BoolOrString::String(value) => value.parse().map_err(serde::de::Error::custom),
    }
}

/// `true`, for `#[serde(default = "crate::serde_util::default_true")]`.
pub(crate) fn default_true() -> bool {
    true
}

/// Serde helpers for comma-separated query parameters (`item__in=1,2,3`).
///
/// Use as `#[serde(with = "crate::serde_util::comma")]` on `Vec<T>` fields.
pub(crate) mod comma {
    use std::fmt::{self, Display};
    use std::str::FromStr;

    use serde::de::{self, Deserialize, Deserializer};
    use serde::ser::Serializer;

    struct Joined<'a, T>(&'a [T]);

    impl<T: Display> Display for Joined<'_, T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for (index, value) in self.0.iter().enumerate() {
                if index > 0 {
                    f.write_str(",")?;
                }
                value.fmt(f)?;
            }
            Ok(())
        }
    }

    pub(crate) fn serialize<S: Serializer, T: Display>(
        values: &[T],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&Joined(values))
    }

    pub(crate) fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        T::Err: Display,
    {
        let raw = String::deserialize(deserializer)?;
        if raw.is_empty() {
            return Ok(Vec::new());
        }
        raw.split(',')
            .map(|value| value.parse().map_err(de::Error::custom))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[expect(
        clippy::option_option,
        reason = "mirrors the tri-state PATCH fields under test"
    )]
    struct Patch {
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "super::double_option"
        )]
        value: Option<Option<u32>>,
    }

    #[test]
    fn double_option_roundtrips() {
        for (patch, json) in [
            (Patch { value: None }, "{}"),
            (Patch { value: Some(None) }, r#"{"value":null}"#),
            (
                Patch {
                    value: Some(Some(3)),
                },
                r#"{"value":3}"#,
            ),
        ] {
            assert_eq!(serde_json::to_string(&patch).unwrap(), json);
            assert_eq!(serde_json::from_str::<Patch>(json).unwrap(), patch);
        }
    }
}
