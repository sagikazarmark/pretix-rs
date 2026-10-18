//! Sub-events: the dates of an event series.
//!
//! <https://docs.pretix.eu/dev/api/resources/subevents.html>

use std::collections::BTreeMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::{AttrFilters, Ordering};
use crate::types::{Decimal, LocalizedString, string_enum};
use crate::{Client, EventScope, OrganizerScope, Page, Request};

/// A date of an event series (sub-event).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct SubEvent {
    /// Internal ID of the sub-event.
    pub id: u64,
    /// The sub-event's full name.
    pub name: LocalizedString,
    /// Slug of the parent event. Missing in create responses according to the pretix
    /// documentation.
    #[serde(default)]
    pub event: Option<String>,
    /// Whether the sub-event's ticket shop is publicly available.
    pub active: bool,
    /// Whether the sub-event is shown in public lists.
    pub is_public: bool,
    /// Start of the sub-event.
    pub date_from: Timestamp,
    /// End of the sub-event.
    pub date_to: Option<Timestamp>,
    /// Admission time.
    pub date_admission: Option<Timestamp>,
    /// When the ticket shop opens.
    pub presale_start: Option<Timestamp>,
    /// When the ticket shop closes.
    pub presale_end: Option<Timestamp>,
    /// The description of the sub-event.
    #[serde(default)]
    pub frontpage_text: Option<LocalizedString>,
    /// The sub-event location.
    pub location: Option<LocalizedString>,
    /// Latitude of the location.
    pub geo_lat: Option<f64>,
    /// Longitude of the location.
    pub geo_lon: Option<f64>,
    /// Items for which this sub-event overrides the default price or settings.
    #[serde(default)]
    pub item_price_overrides: Vec<SubEventItemPriceOverride>,
    /// Variations for which this sub-event overrides the default price or settings.
    #[serde(default)]
    pub variation_price_overrides: Vec<SubEventVariationPriceOverride>,
    /// Values of organizer-defined meta data properties.
    #[serde(default)]
    pub meta_data: BTreeMap<String, String>,
    /// ID of the seating plan, if reserved seating is used.
    pub seating_plan: Option<u64>,
    /// Maps seating plan categories to item IDs.
    #[serde(default)]
    pub seat_category_mapping: BTreeMap<String, Option<u64>>,
    /// Internal comment (not in the pretix docs).
    #[serde(default)]
    pub comment: Option<String>,
    /// Last modification of this object.
    #[serde(default)]
    pub last_modified: Option<Timestamp>,
    /// Availability (100 = available, lower = sold out or reserved, `None` = unknown).
    /// Only included when listing with `with_availability_for`.
    #[serde(default)]
    pub best_availability_state: Option<u32>,
}

/// Price and availability override of an item for a sub-event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct SubEventItemPriceOverride {
    /// The internal item ID.
    pub item: u64,
    /// Whether the item is unavailable for this sub-event.
    #[serde(default)]
    #[builder(default)]
    pub disabled: bool,
    /// Start of availability.
    #[serde(default)]
    pub available_from: Option<Timestamp>,
    /// End of availability.
    #[serde(default)]
    pub available_until: Option<Timestamp>,
    /// The price, or `None` for the default price.
    pub price: Option<Decimal>,
}

/// Price and availability override of a variation for a sub-event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct SubEventVariationPriceOverride {
    /// The internal variation ID.
    pub variation: u64,
    /// Whether the variation is unavailable for this sub-event.
    #[serde(default)]
    #[builder(default)]
    pub disabled: bool,
    /// Start of availability.
    #[serde(default)]
    pub available_from: Option<Timestamp>,
    /// End of availability.
    #[serde(default)]
    pub available_until: Option<Timestamp>,
    /// The price, or `None` for the default price.
    pub price: Option<Decimal>,
}

string_enum! {
    /// Fields sub-events can be sorted by.
    pub enum SubEventOrdering {
        /// By ID.
        Id = "id",
        /// By start (default).
        DateFrom = "date_from",
        /// By last modification.
        LastModified = "last_modified",
    }
}

/// Query parameters for [`SubEvents::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListSubEvents {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<SubEventOrdering>>,
    /// Only sub-events with this `is_public` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    /// Only sub-events with this `active` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Only sub-events that are (`true`) or are not (`false`) current or in the future.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_future: Option<bool>,
    /// Only sub-events that are (`true`) or are not (`false`) over.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_past: Option<bool>,
    /// Only sub-events starting at or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_from_after: Option<Timestamp>,
    /// Only sub-events starting at or before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_from_before: Option<Timestamp>,
    /// Only sub-events ending at or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_to_after: Option<Timestamp>,
    /// Only sub-events ending at or before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_to_before: Option<Timestamp>,
    /// Only sub-events happening during or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_after: Option<Timestamp>,
    /// Full-text search.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub search: Option<String>,
    /// Only sub-events changed since this time (deletions are not reported).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_since: Option<Timestamp>,
    /// Include `best_availability_state` for this sales channel (slow).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub with_availability_for: Option<String>,
    /// Filters on meta data values (`attr[key]=value`), respecting defaults set on
    /// organizer or event level.
    #[serde(flatten)]
    #[builder(default)]
    pub attr: AttrFilters,
}

/// Query parameters for [`OrganizerSubEvents::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListOrganizerSubEvents {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<SubEventOrdering>>,
    /// Only sub-events with this `is_public` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    /// Only sub-events with this `active` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Only sub-events whose parent event has this `live` value (`event__live`).
    #[serde(rename = "event__live", skip_serializing_if = "Option::is_none")]
    pub event_live: Option<bool>,
    /// Only sub-events that are (`true`) or are not (`false`) current or in the future.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_future: Option<bool>,
    /// Only sub-events that are (`true`) or are not (`false`) over.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_past: Option<bool>,
    /// Only sub-events starting at or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_from_after: Option<Timestamp>,
    /// Only sub-events starting at or before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_from_before: Option<Timestamp>,
    /// Only sub-events ending at or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_to_after: Option<Timestamp>,
    /// Only sub-events ending at or before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_to_before: Option<Timestamp>,
    /// Only sub-events happening during or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_after: Option<Timestamp>,
    /// Full-text search (also matches the parent event's slug).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub search: Option<String>,
    /// Only sub-events of events available on this sales channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub sales_channel: Option<String>,
    /// Only sub-events changed since this time (deletions are not reported). Supported
    /// by pretix, though not documented for this endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_since: Option<Timestamp>,
    /// Include `best_availability_state` for this sales channel (slow). Supported by
    /// pretix, though not documented for this endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub with_availability_for: Option<String>,
    /// Filters on meta data values (`attr[key]=value`), respecting defaults set on
    /// organizer or event level. Supported by pretix, though not documented for this
    /// endpoint.
    #[serde(flatten)]
    #[builder(default)]
    pub attr: AttrFilters,
}

/// Body of [`SubEvents::create`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateSubEvent {
    /// The sub-event's full name.
    #[builder(into)]
    pub name: LocalizedString,
    /// Start of the sub-event.
    pub date_from: Timestamp,
    /// Whether the sub-event's ticket shop is publicly available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Whether the sub-event is shown in public lists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    /// End of the sub-event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_to: Option<Timestamp>,
    /// Admission time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_admission: Option<Timestamp>,
    /// When the ticket shop opens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presale_start: Option<Timestamp>,
    /// When the ticket shop closes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presale_end: Option<Timestamp>,
    /// The description of the sub-event.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub frontpage_text: Option<LocalizedString>,
    /// The sub-event location.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub location: Option<LocalizedString>,
    /// Latitude of the location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geo_lat: Option<f64>,
    /// Longitude of the location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geo_lon: Option<f64>,
    /// Items for which this sub-event overrides the default price or settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_price_overrides: Option<Vec<SubEventItemPriceOverride>>,
    /// Variations for which this sub-event overrides the default price or settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation_price_overrides: Option<Vec<SubEventVariationPriceOverride>>,
    /// Values of organizer-defined meta data properties. Always sent (pretix requires
    /// the field), empty by default.
    #[builder(default)]
    pub meta_data: BTreeMap<String, String>,
    /// ID of the seating plan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seating_plan: Option<u64>,
    /// Maps seating plan categories to item IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seat_category_mapping: Option<BTreeMap<String, Option<u64>>>,
    /// Internal comment (not in the pretix docs).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub comment: Option<String>,
}

/// Body of [`SubEvents::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateSubEvent {
    /// The sub-event's full name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<LocalizedString>,
    /// Whether the sub-event's ticket shop is publicly available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Whether the sub-event is shown in public lists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    /// Start of the sub-event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_from: Option<Timestamp>,
    /// End of the sub-event.
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
    /// The description of the sub-event.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub frontpage_text: Option<Option<LocalizedString>>,
    /// The sub-event location.
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
    /// Items for which this sub-event overrides the default price or settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_price_overrides: Option<Vec<SubEventItemPriceOverride>>,
    /// Variations for which this sub-event overrides the default price or settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation_price_overrides: Option<Vec<SubEventVariationPriceOverride>>,
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
    /// Internal comment (not in the pretix docs).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub comment: Option<Option<String>>,
}

/// Endpoints for the dates of an event series. Created with [`EventScope::subevents`].
#[derive(Debug, Clone)]
pub struct SubEvents {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Dates (sub-events) of this event series.
    #[must_use]
    pub fn subevents(&self) -> SubEvents {
        SubEvents {
            client: self.client(),
            path: self.path().join("subevents"),
        }
    }
}

impl SubEvents {
    pub(crate) fn client(&self) -> Client {
        self.client.clone()
    }

    /// `organizers/{organizer}/events/{event}/subevents/`
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// Lists sub-events.
    pub fn list(&self, params: &ListSubEvents) -> Request<Page<SubEvent>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one sub-event.
    pub fn get(&self, id: u64) -> Request<SubEvent> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a sub-event.
    pub fn create(&self, body: &CreateSubEvent) -> Request<SubEvent> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a sub-event.
    pub fn update(&self, id: u64, body: &UpdateSubEvent) -> Request<SubEvent> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a sub-event. Sub-events with orders cannot be deleted.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}

/// List endpoint for the sub-events of all event series of an organizer. Created with
/// [`OrganizerScope::subevents`].
#[derive(Debug, Clone)]
pub struct OrganizerSubEvents {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Sub-events of all event series of this organizer the credentials have access to.
    #[must_use]
    pub fn subevents(&self) -> OrganizerSubEvents {
        OrganizerSubEvents {
            client: self.client(),
            path: self.path().join("subevents"),
        }
    }
}

impl OrganizerSubEvents {
    /// Lists sub-events across all event series.
    pub fn list(&self, params: &ListOrganizerSubEvents) -> Request<Page<SubEvent>> {
        self.client.get(&self.path).query(params).page()
    }
}
