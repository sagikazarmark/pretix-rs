use std::fmt;

use jiff::civil::Date;
use serde::{Deserialize, Serialize};

/// A date range filter: two dates separated by `/` (either side may be open), or the
/// name of a predefined range such as `week_this` or `month_next`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DateRange(String);

impl DateRange {
    /// A range between two dates; `None` leaves that side open.
    #[must_use]
    pub fn between(from: Option<Date>, until: Option<Date>) -> Self {
        let side = |date: Option<Date>| date.map(|d| d.to_string()).unwrap_or_default();
        Self(format!("{}/{}", side(from), side(until)))
    }

    /// A predefined range, e.g. `week_this`, `week_next`, `month_this`.
    pub fn named(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The wire representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DateRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::DateRange;

    #[test]
    fn formats_ranges() {
        assert_eq!(
            DateRange::between(Some(date(2022, 3, 18)), Some(date(2022, 3, 23))).as_str(),
            "2022-03-18/2022-03-23"
        );
        assert_eq!(
            DateRange::between(None, Some(date(2022, 3, 23))).as_str(),
            "/2022-03-23"
        );
        assert_eq!(DateRange::named("week_this").as_str(), "week_this");
    }
}
