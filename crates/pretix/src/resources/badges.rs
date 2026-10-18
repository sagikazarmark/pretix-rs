//! Badge layouts of the bundled badges plugin.
//!
//! <https://docs.pretix.eu/dev/api/resources/badges.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::{Client, EventScope, ListParams, Page, Request};

/// A PDF badge layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BadgeLayout {
    /// Internal layout ID.
    pub id: u64,
    /// Internal description.
    pub name: String,
    /// Whether this is the default layout.
    pub default: bool,
    /// Layout specification for libpretixprint.
    pub layout: serde_json::Value,
    /// URL of the background PDF file.
    pub background: Option<String>,
    /// Products this layout is assigned to.
    #[serde(default)]
    pub item_assignments: Vec<BadgeLayoutAssignment>,
}

/// An assignment of a product to a [`BadgeLayout`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BadgeLayoutAssignment {
    /// Item ID.
    pub item: u64,
}

/// An assignment of a product to a layout, as listed by [`BadgeItems::list`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BadgeItem {
    /// Internal ID of the assignment.
    pub id: u64,
    /// Layout ID.
    pub layout: u64,
    /// Item ID.
    pub item: u64,
}

/// Endpoints for the badge layouts of an event. Created with
/// [`EventScope::badge_layouts`].
#[derive(Debug, Clone)]
pub struct BadgeLayouts {
    client: Client,
    path: Path,
}

/// Endpoint listing the product assignments of all badge layouts of an event. Created
/// with [`EventScope::badge_items`].
#[derive(Debug, Clone)]
pub struct BadgeItems {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Badge layouts of this event.
    #[must_use]
    pub fn badge_layouts(&self) -> BadgeLayouts {
        BadgeLayouts {
            client: self.client(),
            path: self.path().join("badgelayouts"),
        }
    }

    /// Product assignments of the badge layouts of this event.
    #[must_use]
    pub fn badge_items(&self) -> BadgeItems {
        BadgeItems {
            client: self.client(),
            path: self.path().join("badgeitems"),
        }
    }
}

impl BadgeLayouts {
    /// Lists badge layouts.
    pub fn list(&self, params: &ListParams) -> Request<Page<BadgeLayout>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one badge layout.
    pub fn get(&self, id: u64) -> Request<BadgeLayout> {
        self.client.get(&self.path.join(id)).json()
    }
}

impl BadgeItems {
    /// Lists the assignments of products to badge layouts.
    pub fn list(&self, params: &ListParams) -> Request<Page<BadgeItem>> {
        self.client.get(&self.path).query(params).page()
    }
}
