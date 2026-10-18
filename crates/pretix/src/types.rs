//! Data types shared across resources.

mod date_range;
mod decimal;
mod expandable;
mod file;
mod i18n;
mod relative;
mod webhook;

pub use date_range::DateRange;
pub use decimal::Decimal;
pub use expandable::Expandable;
pub use file::FileId;
pub use i18n::LocalizedString;
pub use relative::{RelativeDate, RelativeDateTime};
pub use webhook::WebhookNotification;

/// Defines an enum for a string-valued, enum-like API field.
///
/// pretix may add new values at any time, so every generated enum has an
/// `Unrecognized(String)` variant that preserves unknown values (and serializes them back
/// unchanged). The enum is `#[non_exhaustive]`.
///
/// Generates `as_str`, `Display`, `FromStr`, `From<&str>`, `From<String>`, `AsRef<str>`
/// and serde implementations. With the `schemars` feature, the JSON Schema lists the
/// known values and also accepts any other string.
///
/// ```ignore
/// string_enum! {
///     /// The status of an order.
///     pub enum OrderStatus {
///         /// Pending payment.
///         Pending = "n",
///         /// Paid.
///         Paid = "p",
///     }
/// }
/// ```
macro_rules! string_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $value:literal
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[non_exhaustive]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                #[cfg_attr(feature = "schemars", schemars(rename = $value))]
                $variant,
            )+
            /// A value not known to this version of the crate.
            ///
            /// Do not construct this for known values; use `From<&str>` instead.
            #[cfg_attr(
                feature = "schemars",
                schemars(untagged, description = "A value not known to this version of the crate.")
            )]
            Unrecognized(::std::string::String),
        }

        impl $name {
            /// The value as sent over the wire.
            pub fn as_str(&self) -> &str {
                match self {
                    $( Self::$variant => $value, )+
                    Self::Unrecognized(value) => value.as_str(),
                }
            }
        }

        impl ::core::convert::From<&str> for $name {
            fn from(value: &str) -> Self {
                match value {
                    $( $value => Self::$variant, )+
                    other => Self::Unrecognized(other.to_owned()),
                }
            }
        }

        impl ::core::convert::From<::std::string::String> for $name {
            fn from(value: ::std::string::String) -> Self {
                match value.as_str() {
                    $( $value => Self::$variant, )+
                    _ => Self::Unrecognized(value),
                }
            }
        }

        impl ::core::str::FromStr for $name {
            type Err = ::core::convert::Infallible;

            fn from_str(value: &str) -> ::core::result::Result<Self, Self::Err> {
                Ok(Self::from(value))
            }
        }

        impl ::core::convert::AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> ::core::result::Result<Self, D::Error> {
                <::std::string::String as ::serde::Deserialize>::deserialize(deserializer).map(Self::from)
            }
        }
    };
}

pub(crate) use string_enum;

#[cfg(test)]
#[allow(unreachable_pub)]
mod tests {
    string_enum! {
        /// Test enum.
        pub enum Status {
            /// Paid.
            Paid = "p",
            /// Pending.
            Pending = "n",
        }
    }

    #[test]
    fn known_values() {
        let status: Status = serde_json::from_str(r#""p""#).unwrap();
        assert_eq!(status, Status::Paid);
        assert_eq!(serde_json::to_string(&status).unwrap(), r#""p""#);
        assert_eq!(status.to_string(), "p");
    }

    #[test]
    fn unknown_values_roundtrip() {
        let status: Status = serde_json::from_str(r#""x""#).unwrap();
        assert_eq!(status, Status::Unrecognized("x".into()));
        assert_eq!(serde_json::to_string(&status).unwrap(), r#""x""#);
        assert_eq!(Status::from("n"), Status::Pending);
    }
}
