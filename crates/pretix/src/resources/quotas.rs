//! Quotas: how many times items can be sold.
//!
//! <https://docs.pretix.eu/dev/api/resources/quotas.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Ordering;
use crate::types::string_enum;
use crate::{Client, EventScope, Page, Request};

/// A quota.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct Quota {
    /// Internal ID of the quota.
    pub id: u64,
    /// The internal name of the quota.
    pub name: String,
    /// The size of the quota, `None` for unlimited.
    pub size: Option<u64>,
    /// IDs of the items this quota acts on.
    #[serde(default)]
    pub items: Vec<u64>,
    /// IDs of the item variations this quota acts on.
    #[serde(default)]
    pub variations: Vec<u64>,
    /// ID of the date inside an event series this quota belongs to.
    pub subevent: Option<u64>,
    /// Whether the quota closes as soon as it is sold out once.
    pub close_when_sold_out: bool,
    /// Whether the quota is currently closed.
    pub closed: bool,
    /// Whether the quota regains capacity as soon as tickets are scanned at an exit.
    #[serde(default)]
    pub release_after_exit: bool,
    /// Whether the quota is ignored when calculating the event's availability.
    pub ignore_for_event_availability: bool,
    /// Whether the quota is available. Only included when requested with availability;
    /// may be slightly out of date.
    #[serde(default)]
    pub available: Option<bool>,
    /// Number of available tickets (`None` = unlimited or not requested). Only included
    /// when requested with availability; may be slightly out of date.
    #[serde(default)]
    pub available_number: Option<u64>,
}

/// Availability information of a quota, returned by [`Quotas::availability`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct QuotaAvailability {
    /// Whether the quota is available.
    pub available: bool,
    /// Number of available tickets, `None` for unlimited quotas.
    pub available_number: Option<u64>,
    /// The size of the quota, `None` for unlimited quotas.
    pub total_size: Option<u64>,
    /// Number of tickets in pending orders.
    pub pending_orders: u64,
    /// Number of tickets in paid orders.
    pub paid_orders: u64,
    /// Number of tickets scanned at an exit.
    pub exited_orders: u64,
    /// Number of tickets in carts.
    pub cart_positions: u64,
    /// Number of tickets blocked by vouchers.
    pub blocking_vouchers: u64,
    /// Number of people on the waiting list.
    pub waiting_list: u64,
}

string_enum! {
    /// Fields quotas can be sorted by.
    pub enum QuotaOrdering {
        /// By ID (default).
        Id = "id",
        /// By size.
        Size = "size",
    }
}

/// Query parameters for [`Quotas::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListQuotas {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<QuotaOrdering>>,
    /// Only quotas of this sub-event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// Only quotas of one of these sub-events.
    #[serde(
        rename = "subevent__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub subevent_in: Vec<u64>,
    /// Only quotas that include one of these items.
    #[serde(
        rename = "items__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub items_in: Vec<u64>,
    /// Include `available` and `available_number` (slow).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_availability: Option<bool>,
}

/// Body of [`Quotas::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateQuota {
    /// The internal name of the quota.
    #[builder(into)]
    pub name: String,
    /// The size of the quota, `None` for unlimited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// IDs of the items this quota acts on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<u64>>,
    /// IDs of the item variations this quota acts on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variations: Option<Vec<u64>>,
    /// ID of the date inside an event series this quota belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// Whether the quota closes as soon as it is sold out once.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_when_sold_out: Option<bool>,
    /// Whether the quota is closed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
    /// Whether the quota regains capacity as soon as tickets are scanned at an exit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_after_exit: Option<bool>,
    /// Whether the quota is ignored when calculating the event's availability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_for_event_availability: Option<bool>,
}

/// Body of [`Quotas::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateQuota {
    /// The internal name of the quota.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<String>,
    /// The size of the quota, `Some(None)` for unlimited.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub size: Option<Option<u64>>,
    /// IDs of the items this quota acts on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<u64>>,
    /// IDs of the item variations this quota acts on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variations: Option<Vec<u64>>,
    /// ID of the date inside an event series this quota belongs to.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub subevent: Option<Option<u64>>,
    /// Whether the quota closes as soon as it is sold out once.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_when_sold_out: Option<bool>,
    /// Whether the quota is closed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
    /// Whether the quota regains capacity as soon as tickets are scanned at an exit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_after_exit: Option<bool>,
    /// Whether the quota is ignored when calculating the event's availability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_for_event_availability: Option<bool>,
}

/// Query parameters of [`Quotas::get_with`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct GetQuota {
    /// Include `available` and `available_number` (slow).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_availability: Option<bool>,
}

/// Endpoints for the quotas of an event. Created with [`EventScope::quotas`].
#[derive(Debug, Clone)]
pub struct Quotas {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Quotas of this event.
    #[must_use]
    pub fn quotas(&self) -> Quotas {
        Quotas {
            client: self.client(),
            path: self.path().join("quotas"),
        }
    }
}

impl Quotas {
    /// Lists quotas.
    pub fn list(&self, params: &ListQuotas) -> Request<Page<Quota>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one quota.
    pub fn get(&self, id: u64) -> Request<Quota> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Fetches one quota with additional query parameters.
    pub fn get_with(&self, id: u64, params: &GetQuota) -> Request<Quota> {
        self.client.get(&self.path.join(id)).query(params).json()
    }

    /// Creates a quota.
    pub fn create(&self, body: &CreateQuota) -> Request<Quota> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a quota.
    pub fn update(&self, id: u64, body: &UpdateQuota) -> Request<Quota> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a quota. The items it acts on might no longer be available for sale.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }

    /// Fetches detailed availability information of a quota.
    pub fn availability(&self, id: u64) -> Request<QuotaAvailability> {
        self.client
            .get(&self.path.join(id).join("availability"))
            .json()
    }
}
