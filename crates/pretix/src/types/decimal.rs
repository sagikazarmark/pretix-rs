use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A decimal number, used for money amounts, tax rates and percentages.
///
/// pretix transmits decimals as strings (`"23.42"`) to avoid floating point rounding.
/// This type keeps the exact value and scale, serializes as a string and accepts
/// strings as well as JSON numbers when deserializing.
///
/// It dereferences to [`rust_decimal::Decimal`] for arithmetic.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Decimal(pub rust_decimal::Decimal);

impl Decimal {
    /// Zero.
    pub const ZERO: Self = Self(rust_decimal::Decimal::ZERO);

    /// Returns the inner [`rust_decimal::Decimal`].
    #[must_use]
    pub fn into_inner(self) -> rust_decimal::Decimal {
        self.0
    }
}

impl Deref for Decimal {
    type Target = rust_decimal::Decimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<rust_decimal::Decimal> for Decimal {
    fn from(value: rust_decimal::Decimal) -> Self {
        Self(value)
    }
}

impl From<Decimal> for rust_decimal::Decimal {
    fn from(value: Decimal) -> Self {
        value.0
    }
}

impl From<i64> for Decimal {
    fn from(value: i64) -> Self {
        Self(value.into())
    }
}

impl FromStr for Decimal {
    type Err = rust_decimal::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        rust_decimal::Decimal::from_str_exact(value.trim()).map(Self)
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for Decimal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct DecimalVisitor;

        impl Visitor<'_> for DecimalVisitor {
            type Value = Decimal;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a decimal number or a string containing one")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Decimal, E> {
                value.parse().map_err(E::custom)
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Decimal, E> {
                Ok(Decimal(value.into()))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Decimal, E> {
                Ok(Decimal(value.into()))
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Decimal, E> {
                // Go through the shortest string representation to avoid binary
                // floating point artifacts (0.1 → 0.1000000000000000055…).
                value.to_string().parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_any(DecimalVisitor)
    }
}

/// Serializes as a string; deserializes from a string or a JSON number.
#[cfg(feature = "schemars")]
impl schemars::JsonSchema for Decimal {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Decimal".into()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        concat!(module_path!(), "::Decimal").into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let description = "A decimal number, transmitted as a string to avoid rounding.";
        if generator.contract().is_serialize() {
            schemars::json_schema!({
                "description": description,
                "type": "string",
                "pattern": r"^-?[0-9]+(\.[0-9]+)?$",
            })
        } else {
            schemars::json_schema!({
                "description": description,
                "type": ["string", "number"],
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Decimal;

    #[test]
    fn keeps_scale() {
        let value: Decimal = serde_json::from_str(r#""23.00""#).unwrap();
        assert_eq!(serde_json::to_string(&value).unwrap(), r#""23.00""#);
    }

    #[test]
    fn accepts_numbers() {
        let value: Decimal = serde_json::from_str("19.5").unwrap();
        assert_eq!(value.to_string(), "19.5");
        let value: Decimal = serde_json::from_str("3").unwrap();
        assert_eq!(value.to_string(), "3");
    }

    #[test]
    fn rejects_garbage() {
        assert!(serde_json::from_str::<Decimal>(r#""abc""#).is_err());
    }
}
