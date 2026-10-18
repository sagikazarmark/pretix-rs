use serde::{Deserialize, Serialize};

/// A reference that is either an ID or, if requested with `expand`, the full object.
///
/// ```json
/// 42                      // Expandable::Id(42)
/// {"id": 42, "name": …}   // Expandable::Expanded(..)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Expandable<I, T> {
    /// The ID (or other identifier) of the referenced object.
    Id(I),
    /// The full object.
    Expanded(Box<T>),
}

impl<I, T> Expandable<I, T> {
    /// The ID, if not expanded.
    pub fn as_id(&self) -> Option<&I> {
        match self {
            Self::Id(id) => Some(id),
            Self::Expanded(_) => None,
        }
    }

    /// The object, if expanded.
    pub fn as_expanded(&self) -> Option<&T> {
        match self {
            Self::Id(_) => None,
            Self::Expanded(value) => Some(value),
        }
    }

    /// The object, if expanded.
    pub fn into_expanded(self) -> Option<T> {
        match self {
            Self::Id(_) => None,
            Self::Expanded(value) => Some(*value),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::Expandable;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Thing {
        id: u64,
        name: String,
    }

    #[test]
    fn deserializes_both_forms() {
        let id: Expandable<u64, Thing> = serde_json::from_str("42").unwrap();
        assert_eq!(id.as_id(), Some(&42));
        let thing: Expandable<u64, Thing> =
            serde_json::from_str(r#"{"id":42,"name":"x"}"#).unwrap();
        assert_eq!(thing.as_expanded().map(|t| t.name.as_str()), Some("x"));
        assert_eq!(
            serde_json::to_string(&thing).unwrap(),
            r#"{"id":42,"name":"x"}"#
        );
    }
}
