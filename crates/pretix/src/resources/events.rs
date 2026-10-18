//! Events.
//!
//! <https://docs.pretix.eu/dev/api/resources/events.html>

use std::collections::BTreeMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::{AttrFilters, Ordering};
use crate::resources::{Settings, SettingsQuery};
use crate::types::{LocalizedString, string_enum};
use crate::{Client, OrganizerScope, Page, Request};

/// An event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Event {
    /// The event's full name.
    pub name: LocalizedString,
    /// Short form of the name, used in URLs.
    pub slug: String,
    /// Whether the ticket shop is publicly available.
    pub live: bool,
    /// Whether the ticket shop is in test mode.
    pub testmode: bool,
    /// ISO 4217 currency code.
    pub currency: String,
    /// Start of the event.
    pub date_from: Timestamp,
    /// End of the event.
    pub date_to: Option<Timestamp>,
    /// Admission time.
    pub date_admission: Option<Timestamp>,
    /// Whether the event shows up in public lists.
    pub is_public: bool,
    /// When the ticket shop opens.
    pub presale_start: Option<Timestamp>,
    /// When the ticket shop closes.
    pub presale_end: Option<Timestamp>,
    /// The event location.
    pub location: Option<LocalizedString>,
    /// Latitude of the location.
    pub geo_lat: Option<f64>,
    /// Longitude of the location.
    pub geo_lon: Option<f64>,
    /// Whether this is an event series (with sub-events). Cannot change after creation.
    pub has_subevents: bool,
    /// Values of organizer-defined meta data properties.
    #[serde(default)]
    pub meta_data: BTreeMap<String, String>,
    /// Package names of the enabled plugins.
    #[serde(default)]
    pub plugins: Vec<String>,
    /// ID of the seating plan, if reserved seating is used.
    pub seating_plan: Option<u64>,
    /// Maps seating plan categories to item IDs.
    #[serde(default)]
    pub seat_category_mapping: BTreeMap<String, Option<u64>>,
    /// Time zone name, e.g. `Europe/Berlin`.
    pub timezone: String,
    /// Item meta data properties and their default values.
    #[serde(default)]
    pub item_meta_properties: BTreeMap<String, String>,
    /// Public keys for non-default ticket signature schemes. Only included in detail
    /// responses.
    #[serde(default)]
    pub valid_keys: Option<BTreeMap<String, Vec<String>>>,
    /// Whether the event is available on all sales channels.
    pub all_sales_channels: bool,
    /// Sales channel identifiers the event is available on, if not all.
    #[serde(default)]
    pub limit_sales_channels: Vec<String>,
    /// **Deprecated**: use `all_sales_channels` and `limit_sales_channels`.
    #[serde(default)]
    pub sales_channels: Vec<String>,
    /// Public, customer-facing URL.
    #[serde(default)]
    pub public_url: Option<String>,
    /// Availability (100 = available, lower = sold out or reserved, `None` = unknown).
    /// Only included when listing with `with_availability_for`.
    #[serde(default)]
    pub best_availability_state: Option<u32>,
}

string_enum! {
    /// Fields events can be sorted by.
    pub enum EventOrdering {
        /// By start date.
        DateFrom = "date_from",
        /// By slug (default).
        Slug = "slug",
    }
}

/// Query parameters for [`Events::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct ListEvents {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<EventOrdering>>,
    /// Only events with this `is_public` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    /// Only events with this `live` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live: Option<bool>,
    /// Only events with this `testmode` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testmode: Option<bool>,
    /// Only events with this `has_subevents` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_subevents: Option<bool>,
    /// Only events that are (`true`) or are not (`false`) current or in the future.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_future: Option<bool>,
    /// Only events that are (`true`) or are not (`false`) over.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_past: Option<bool>,
    /// Only events starting at or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_from_after: Option<Timestamp>,
    /// Only events starting at or before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_from_before: Option<Timestamp>,
    /// Only events ending at or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_to_after: Option<Timestamp>,
    /// Only events ending at or before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_to_before: Option<Timestamp>,
    /// Only events happening during or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_after: Option<Timestamp>,
    /// Only events that can be sold on this sales channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub sales_channel: Option<String>,
    /// Include `best_availability_state` for this sales channel (slow).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub with_availability_for: Option<String>,
    /// Full-text search.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub search: Option<String>,
    /// Filters on meta data values (`attr[key]=value`).
    #[serde(flatten)]
    #[builder(default)]
    pub attr: AttrFilters,
}

/// Body of [`Events::create`], [`Events::create_from`] and [`Events::clone_event`].
///
/// Events cannot be created live.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateEvent {
    /// The event's full name.
    #[builder(into)]
    pub name: LocalizedString,
    /// Short form of the name, used in URLs.
    #[builder(into)]
    pub slug: String,
    /// ISO 4217 currency code.
    #[builder(into)]
    pub currency: String,
    /// Start of the event.
    pub date_from: Timestamp,
    /// End of the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_to: Option<Timestamp>,
    /// Admission time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_admission: Option<Timestamp>,
    /// Whether the shop is in test mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testmode: Option<bool>,
    /// Whether the event shows up in public lists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    /// When the ticket shop opens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presale_start: Option<Timestamp>,
    /// When the ticket shop closes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presale_end: Option<Timestamp>,
    /// The event location.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub location: Option<LocalizedString>,
    /// Latitude of the location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geo_lat: Option<f64>,
    /// Longitude of the location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geo_lon: Option<f64>,
    /// Whether this is an event series.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_subevents: Option<bool>,
    /// Values of organizer-defined meta data properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data: Option<BTreeMap<String, String>>,
    /// ID of the seating plan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seating_plan: Option<u64>,
    /// Maps seating plan categories to item IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seat_category_mapping: Option<BTreeMap<String, Option<u64>>>,
    /// Time zone name, e.g. `Europe/Berlin`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub timezone: Option<String>,
    /// Item meta data properties and their default values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_meta_properties: Option<BTreeMap<String, String>>,
    /// Package names of the plugins to enable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Vec<String>>,
    /// Whether the event is available on all sales channels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_sales_channels: Option<bool>,
    /// Sales channel identifiers the event is available on, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_sales_channels: Option<Vec<String>>,
}

/// Body of [`Events::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateEvent {
    /// The event's full name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<LocalizedString>,
    /// Short form of the name, used in URLs.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub slug: Option<String>,
    /// Whether the shop is live.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live: Option<bool>,
    /// Whether the shop is in test mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testmode: Option<bool>,
    /// ISO 4217 currency code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub currency: Option<String>,
    /// Start of the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_from: Option<Timestamp>,
    /// End of the event.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub date_to: Option<Option<Timestamp>>,
    /// Admission time.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub date_admission: Option<Option<Timestamp>>,
    /// Whether the event shows up in public lists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    /// When the ticket shop opens.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub presale_start: Option<Option<Timestamp>>,
    /// When the ticket shop closes.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub presale_end: Option<Option<Timestamp>>,
    /// The event location.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub location: Option<Option<LocalizedString>>,
    /// Latitude of the location.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub geo_lat: Option<Option<f64>>,
    /// Longitude of the location.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub geo_lon: Option<Option<f64>>,
    /// Values of organizer-defined meta data properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data: Option<BTreeMap<String, String>>,
    /// ID of the seating plan.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub seating_plan: Option<Option<u64>>,
    /// Maps seating plan categories to item IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seat_category_mapping: Option<BTreeMap<String, Option<u64>>>,
    /// Time zone name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub timezone: Option<String>,
    /// Item meta data properties and their default values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_meta_properties: Option<BTreeMap<String, String>>,
    /// Package names of the enabled plugins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Vec<String>>,
    /// Whether the event is available on all sales channels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_sales_channels: Option<bool>,
    /// Sales channel identifiers the event is available on, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_sales_channels: Option<Vec<String>>,
}

#[derive(Serialize)]
struct CloneFrom<'a> {
    clone_from: &'a str,
}

/// Endpoints for the events of an organizer. Created with [`OrganizerScope::events`].
#[derive(Debug, Clone)]
pub struct Events {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Events of this organizer.
    #[must_use]
    pub fn events(&self) -> Events {
        Events {
            client: self.client(),
            path: self.path().join("events"),
        }
    }
}

impl Events {
    /// Lists events.
    pub fn list(&self, params: &ListEvents) -> Request<Page<Event>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one event.
    pub fn get(&self, slug: &str) -> Request<Event> {
        self.client.get(&self.path.join(slug)).json()
    }

    /// Creates an event.
    pub fn create(&self, body: &CreateEvent) -> Request<Event> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Creates an event, copying settings, products, quotas etc. from `source`.
    ///
    /// `source` is either an event slug of the same organizer or
    /// `organizer_slug/event_slug`.
    pub fn create_from(&self, source: &str, body: &CreateEvent) -> Request<Event> {
        self.client
            .post(&self.path)
            .query(&CloneFrom { clone_from: source })
            .json_body(body)
            .json()
    }

    /// Creates a copy of the event `slug` (same organizer only) with the properties
    /// given in `body`.
    pub fn clone_event(&self, slug: &str, body: &CreateEvent) -> Request<Event> {
        self.client
            .post(&self.path.join(slug).join("clone"))
            .json_body(body)
            .json()
    }

    /// Updates an event.
    pub fn update(&self, slug: &str, body: &UpdateEvent) -> Request<Event> {
        self.client
            .patch(&self.path.join(slug))
            .json_body(body)
            .json()
    }

    /// Deletes an event. Events with orders cannot be deleted.
    pub fn delete(&self, slug: &str) -> Request<()> {
        self.client.delete(&self.path.join(slug)).empty()
    }

    /// Fetches the event's settings.
    pub fn settings(&self, slug: &str) -> Request<Settings> {
        self.client
            .get(&self.path.join(slug).join("settings"))
            .json()
    }

    /// Fetches the event's settings with labels and help texts.
    pub fn settings_explained(&self, slug: &str) -> Request<Settings> {
        self.client
            .get(&self.path.join(slug).join("settings"))
            .query(&SettingsQuery { explain: true })
            .json()
    }

    /// Updates settings. Only send the keys you want to set on event level; `null`
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
