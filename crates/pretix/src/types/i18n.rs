use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A multi-lingual string, mapping language codes to translations.
///
/// ```json
/// {"en": "red", "de": "rot", "de-informal": "rot"}
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LocalizedString(pub BTreeMap<String, String>);

impl LocalizedString {
    /// Creates an empty string.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a translation.
    #[must_use]
    pub fn with(mut self, language: impl Into<String>, text: impl Into<String>) -> Self {
        self.0.insert(language.into(), text.into());
        self
    }

    /// The translation for exactly `language`.
    pub fn get(&self, language: &str) -> Option<&str> {
        self.0.get(language).map(String::as_str)
    }

    /// The best available translation: `language`, then its base language (`de` for
    /// `de-informal`), then English, then any translation.
    #[must_use]
    pub fn localize(&self, language: &str) -> Option<&str> {
        let base = language.split(['-', '_']).next().unwrap_or(language);
        self.get(language)
            .or_else(|| self.get(base))
            .or_else(|| self.get("en"))
            .or_else(|| self.0.values().next().map(String::as_str))
    }

    /// Whether no translation is set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<K: Into<String>, V: Into<String>> FromIterator<(K, V)> for LocalizedString {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl<K: Into<String>, V: Into<String>, const N: usize> From<[(K, V); N]> for LocalizedString {
    fn from(entries: [(K, V); N]) -> Self {
        entries.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::LocalizedString;

    #[test]
    fn localizes_with_fallback() {
        let s = LocalizedString::from([("de", "rot"), ("en", "red")]);
        assert_eq!(s.localize("de-informal"), Some("rot"));
        assert_eq!(s.localize("fr"), Some("red"));
        assert_eq!(LocalizedString::new().localize("en"), None);
    }
}
