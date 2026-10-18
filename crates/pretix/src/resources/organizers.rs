//! Organizers: the accounts that run events.
//!
//! <https://docs.pretix.eu/dev/api/resources/organizers.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Ordering;
use crate::resources::{Settings, SettingsQuery};
use crate::types::string_enum;
use crate::{Client, Page, Request};

/// An organizer account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Organizer {
    /// Full name of the organization or company.
    pub name: String,
    /// Short form of the name, used in URLs.
    pub slug: String,
    /// Public, customer-facing URL listing the organizer's events.
    #[serde(default)]
    pub public_url: Option<String>,
    /// Package names of the plugins enabled on organizer level.
    #[serde(default)]
    pub plugins: Vec<String>,
}

string_enum! {
    /// Fields organizers can be sorted by.
    pub enum OrganizerOrdering {
        /// By slug (default).
        Slug = "slug",
        /// By name.
        Name = "name",
    }
}

/// Query parameters for [`Organizers::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct ListOrganizers {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<OrganizerOrdering>>,
}

/// Body of [`Organizers::update`]. Only `plugins` can be changed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateOrganizer {
    /// Package names of the plugins to enable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Vec<String>>,
}

/// Endpoints for organizer accounts. Created with [`Client::organizers`].
#[derive(Debug, Clone)]
pub struct Organizers {
    client: Client,
    path: Path,
}

impl Client {
    /// Organizer accounts the credentials have access to.
    #[must_use]
    pub fn organizers(&self) -> Organizers {
        Organizers {
            client: self.clone(),
            path: Path::root().join("organizers"),
        }
    }
}

impl Organizers {
    /// Lists all organizers the credentials have access to.
    pub fn list(&self, params: &ListOrganizers) -> Request<Page<Organizer>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one organizer.
    pub fn get(&self, slug: &str) -> Request<Organizer> {
        self.client.get(&self.path.join(slug)).json()
    }

    /// Updates an organizer.
    pub fn update(&self, slug: &str, body: &UpdateOrganizer) -> Request<Organizer> {
        self.client
            .patch(&self.path.join(slug))
            .json_body(body)
            .json()
    }

    /// Fetches the organizer's settings.
    pub fn settings(&self, slug: &str) -> Request<Settings> {
        self.client
            .get(&self.path.join(slug).join("settings"))
            .json()
    }

    /// Fetches the organizer's settings with labels and help texts
    /// (`{"key": {"value": …, "label": …, "readonly": …, "help_text": …}}`).
    pub fn settings_explained(&self, slug: &str) -> Request<Settings> {
        self.client
            .get(&self.path.join(slug).join("settings"))
            .query(&SettingsQuery { explain: true })
            .json()
    }

    /// Updates settings. Only send the keys you want to set on organizer level; `null`
    /// unsets a value. Returns all settings.
    pub fn update_settings<B: Serialize + ?Sized>(
        &self,
        slug: &str,
        settings: &B,
    ) -> Request<Settings> {
        self.client
            .patch(&self.path.join(slug).join("settings"))
            .json_body(settings)
            .json()
    }
}
