//! Check-in lists: configuration and statistics of check-ins, and the order positions
//! on a list.
//!
//! <https://docs.pretix.eu/dev/api/resources/checkinlists.html>

use std::collections::BTreeMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::{Expand, FieldSelection, Ordering};
use crate::resources::checkin::{
    CheckinErrorReason, CheckinPosition, CheckinPositionExpand, CheckinPositionOrdering,
    CheckinResult, CheckinType, decode_checkin_result,
};
use crate::resources::orders::OrderStatus;
use crate::resources::reusable_media::MediumType;
use crate::resources::subevents::SubEvent;
use crate::types::{Expandable, string_enum};
use crate::{Client, EventScope, Page, Request};

/// A check-in list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CheckinList {
    /// Internal ID of the check-in list.
    pub id: u64,
    /// Internal name of the check-in list.
    pub name: String,
    /// Whether the list contains tickets of all products (`limit_products` is ignored).
    pub all_products: bool,
    /// Item IDs included in this list.
    #[serde(default)]
    pub limit_products: Vec<u64>,
    /// The event series date this list belongs to.
    pub subevent: Option<Expandable<u64, SubEvent>>,
    /// Number of tickets that match this list.
    #[serde(default)]
    pub position_count: Option<u64>,
    /// Number of check-ins performed on this list.
    #[serde(default)]
    pub checkin_count: Option<u64>,
    /// Whether the list also contains tickets from pending orders.
    pub include_pending: bool,
    /// Whether subsequent scans are stored as additional check-ins instead of warnings.
    pub allow_multiple_entries: bool,
    /// Whether subsequent scans are valid if the last scan was an exit.
    pub allow_entry_after_exit: bool,
    /// Custom check-in logic (not a stable API).
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub rules: serde_json::Map<String, serde_json::Value>,
    /// When everyone is automatically checked out.
    #[serde(default)]
    pub exit_all_at: Option<Timestamp>,
    /// Whether add-on tickets can be redeemed by scanning their parent ticket.
    pub addon_match: bool,
    /// Whether check-ins on this list are ignored in most reporting features.
    #[serde(default)]
    pub ignore_in_statistics: bool,
    /// Whether tickets checked in on this list are considered "used" (default `true`).
    #[serde(default = "crate::serde_util::default_true")]
    pub consider_tickets_used: bool,
}

/// Detailed status of a check-in list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CheckinListStatus {
    /// Number of check-ins.
    pub checkin_count: u64,
    /// Number of tickets on the list.
    pub position_count: u64,
    /// Number of attendees currently inside.
    pub inside_count: u64,
    /// The event.
    pub event: CheckinListStatusEvent,
    /// Statistics per item.
    #[serde(default)]
    pub items: Vec<CheckinListStatusItem>,
}

/// The event in a [`CheckinListStatus`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CheckinListStatusEvent {
    /// Name of the event.
    pub name: String,
}

/// Statistics of one item in a [`CheckinListStatus`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CheckinListStatusItem {
    /// Name of the item.
    pub name: String,
    /// Internal ID of the item.
    pub id: u64,
    /// Number of check-ins.
    pub checkin_count: u64,
    /// Whether the item grants admission.
    pub admission: bool,
    /// Number of tickets.
    pub position_count: u64,
    /// Statistics per variation.
    #[serde(default)]
    pub variations: Vec<CheckinListStatusVariation>,
}

/// Statistics of one variation in a [`CheckinListStatusItem`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CheckinListStatusVariation {
    /// Name of the variation.
    pub value: String,
    /// Internal ID of the variation.
    pub id: u64,
    /// Number of check-ins.
    pub checkin_count: u64,
    /// Number of tickets.
    pub position_count: u64,
}

string_enum! {
    /// Fields check-in lists can be sorted by.
    pub enum CheckinListOrdering {
        /// By ID.
        Id = "id",
        /// By name.
        Name = "name",
        /// By start of the sub-event.
        SubeventDateFrom = "subevent__date_from",
    }
}

string_enum! {
    /// Fields of check-in lists that can be expanded into full objects.
    pub enum CheckinListExpand {
        /// The sub-event.
        Subevent = "subevent",
    }
}

/// Query parameters for [`CheckinLists::list`].
///
/// To exclude response fields (e.g. `checkin_count`, for performance), use
/// [`CheckinLists::list_with`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListCheckinLists {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order. Default: `subevent__date_from,name`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<CheckinListOrdering>>,
    /// Only lists of this sub-event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// Only lists valid for this sub-event (including lists for all sub-events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent_match: Option<u64>,
    /// Exclude lists of sub-events that are over at this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_after: Option<Timestamp>,
    /// Fields to expand into full objects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub expand: Vec<CheckinListExpand>,
}

/// Query parameters for [`CheckinListPositions::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListCheckinListPositions {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order. Default: `attendee_name,positionid`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<CheckinPositionOrdering>>,
    /// Return results regardless of the order status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_status: Option<bool>,
    /// Only positions of the order with this code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub order: Option<String>,
    /// Fuzzy search matching the attendee name, order code, invoice address name and
    /// the beginning of the secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub search: Option<String>,
    /// Only positions of this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<u64>,
    /// Only positions of one of these items.
    #[serde(
        rename = "item__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub item_in: Vec<u64>,
    /// Only positions of this variation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// Only positions of one of these variations.
    #[serde(
        rename = "variation__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub variation_in: Vec<u64>,
    /// Only positions with this attendee name (including their add-ons).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub attendee_name: Option<String>,
    /// Only positions with this ticket secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub secret: Option<String>,
    /// Only positions of orders with this status.
    #[serde(rename = "order__status", skip_serializing_if = "Option::is_none")]
    pub order_status: Option<OrderStatus>,
    /// Only positions of orders with one of these statuses.
    #[serde(
        rename = "order__status__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub order_status_in: Vec<OrderStatus>,
    /// Only positions that have (`true`) or have not (`false`) been checked in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_checkin: Option<bool>,
    /// Only positions of this sub-event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// Only positions of one of these sub-events.
    #[serde(
        rename = "subevent__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub subevent_in: Vec<u64>,
    /// Only add-ons to the position with this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addon_to: Option<u64>,
    /// Only add-ons to one of these positions.
    #[serde(
        rename = "addon_to__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub addon_to_in: Vec<u64>,
    /// Only positions with the voucher with this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voucher: Option<u64>,
    /// Only positions with this voucher code.
    #[serde(rename = "voucher__code", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub voucher_code: Option<String>,
}

/// Body of [`CheckinLists::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateCheckinList {
    /// Internal name of the check-in list.
    #[builder(into)]
    pub name: String,
    /// Whether the list contains tickets of all products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_products: Option<bool>,
    /// Item IDs included in this list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_products: Option<Vec<u64>>,
    /// ID of the event series date this list belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// Whether the list also contains tickets from pending orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_pending: Option<bool>,
    /// Whether subsequent scans are stored as additional check-ins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_multiple_entries: Option<bool>,
    /// Whether subsequent scans are valid if the last scan was an exit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_entry_after_exit: Option<bool>,
    /// Custom check-in logic (not a stable API, modifications are discouraged).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<serde_json::Map<String, serde_json::Value>>,
    /// When everyone is automatically checked out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_all_at: Option<Timestamp>,
    /// Whether add-on tickets can be redeemed by scanning their parent ticket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addon_match: Option<bool>,
    /// Whether check-ins on this list are ignored in most reporting features.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_in_statistics: Option<bool>,
    /// Whether tickets checked in on this list are considered "used".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consider_tickets_used: Option<bool>,
}

/// Body of [`CheckinLists::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateCheckinList {
    /// Internal name of the check-in list.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<String>,
    /// Whether the list contains tickets of all products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_products: Option<bool>,
    /// Item IDs included in this list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_products: Option<Vec<u64>>,
    /// ID of the event series date this list belongs to.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub subevent: Option<Option<u64>>,
    /// Whether the list also contains tickets from pending orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_pending: Option<bool>,
    /// Whether subsequent scans are stored as additional check-ins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_multiple_entries: Option<bool>,
    /// Whether subsequent scans are valid if the last scan was an exit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_entry_after_exit: Option<bool>,
    /// Custom check-in logic (not a stable API, modifications are discouraged).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<serde_json::Map<String, serde_json::Value>>,
    /// When everyone is automatically checked out.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub exit_all_at: Option<Option<Timestamp>>,
    /// Whether add-on tickets can be redeemed by scanning their parent ticket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addon_match: Option<bool>,
    /// Whether check-ins on this list are ignored in most reporting features.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_in_statistics: Option<bool>,
    /// Whether tickets checked in on this list are considered "used".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consider_tickets_used: Option<bool>,
}

/// Body of [`CheckinLists::failed_checkins`]: a scan that failed offline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateFailedCheckin {
    /// Why the scan failed.
    pub error_reason: CheckinErrorReason,
    /// The raw barcode or identifier that was scanned.
    #[builder(into)]
    pub raw_barcode: String,
    /// Type of the scanned medium. Defaults to `barcode`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_source_type: Option<MediumType>,
    /// Time of the scan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime: Option<Timestamp>,
    /// Type of scan. Defaults to `entry`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<CheckinType>,
    /// Internal ID of the matched order position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    /// Internal ID of the matched item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_item: Option<u64>,
    /// Internal ID of the matched variation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_variation: Option<u64>,
    /// Internal ID of the matched event series date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_subevent: Option<u64>,
}

/// Body of [`CheckinListPositions::redeem`] and [`CheckinListPositions::redeem_secret`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct RedeemPosition {
    /// Whether the client can handle questions. **Must** be `false` if not.
    /// Defaults to `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub questions_supported: Option<bool>,
    /// **Deprecated**: whether the reason `canceled` may be returned (otherwise
    /// `unpaid`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canceled_supported: Option<bool>,
    /// Time of the check-in. Defaults to now.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime: Option<Timestamp>,
    /// Succeed regardless of revoked codes, previous check-ins or missing answers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    /// Entry (default) or exit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<CheckinType>,
    /// Succeed even if the order is pending (requires `include_pending` on the list).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_unpaid: Option<bool>,
    /// Unique random value identifying this check-in, to make retries idempotent.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub nonce: Option<String>,
    /// Answers by question ID. Choice answers are comma-separated option IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answers: Option<BTreeMap<u64, String>>,
}

/// Endpoints for the check-in lists of an event. Created with
/// [`EventScope::checkin_lists`].
#[derive(Debug, Clone)]
pub struct CheckinLists {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Check-in lists of this event.
    #[must_use]
    pub fn checkin_lists(&self) -> CheckinLists {
        CheckinLists {
            client: self.client(),
            path: self.path().join("checkinlists"),
        }
    }
}

impl CheckinLists {
    /// Lists check-in lists.
    pub fn list(&self, params: &ListCheckinLists) -> Request<Page<CheckinList>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Like [`CheckinLists::list`], excluding the given fields (e.g. `checkin_count` and
    /// `position_count`, for performance). The lists are therefore returned as JSON.
    ///
    /// pretix only supports `exclude` here; [`FieldSelection::include`] is ignored.
    pub fn list_with(
        &self,
        params: &ListCheckinLists,
        fields: &FieldSelection,
    ) -> Request<Page<serde_json::Value>> {
        self.client
            .get(&self.path)
            .query(params)
            .query(fields)
            .page()
    }

    /// Fetches one check-in list.
    pub fn get(&self, id: u64) -> Request<CheckinList> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Fetches detailed status information on a check-in list.
    pub fn status(&self, id: u64) -> Request<CheckinListStatus> {
        self.client.get(&self.path.join(id).join("status")).json()
    }

    /// Creates a check-in list.
    pub fn create(&self, body: &CreateCheckinList) -> Request<CheckinList> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a check-in list.
    pub fn update(&self, id: u64, body: &UpdateCheckinList) -> Request<CheckinList> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a check-in list, **including all check-ins performed on it**.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }

    /// Stores a failed check-in (for statistics, when validating scans offline).
    pub fn failed_checkins(&self, id: u64, body: &CreateFailedCheckin) -> Request<()> {
        self.client
            .post(&self.path.join(id).join("failed_checkins"))
            .json_body(body)
            .empty()
    }

    /// Order positions on the check-in list `list_id`.
    #[must_use]
    pub fn positions(&self, list_id: u64) -> CheckinListPositions {
        CheckinListPositions {
            client: self.client.clone(),
            path: self.path.join(list_id).join("positions"),
        }
    }
}

/// Endpoints for the order positions on a check-in list. Created with
/// [`CheckinLists::positions`].
///
/// For scanning apps, prefer the organizer-level
/// [`CheckinRpc`](crate::resources::checkin::CheckinRpc).
#[derive(Debug, Clone)]
pub struct CheckinListPositions {
    client: Client,
    path: Path,
}

impl CheckinListPositions {
    /// Lists order positions on the list.
    pub fn list(&self, params: &ListCheckinListPositions) -> Request<Page<CheckinPosition>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Like [`CheckinListPositions::list`], expanding fields of the positions (which
    /// are therefore returned as JSON).
    pub fn list_with(
        &self,
        params: &ListCheckinListPositions,
        expand: &Expand<CheckinPositionExpand>,
    ) -> Request<Page<serde_json::Value>> {
        self.client
            .get(&self.path)
            .query(params)
            .query(expand)
            .page()
    }

    /// Fetches one order position on the list.
    pub fn get(&self, id: u64) -> Request<CheckinPosition> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Tries to check in the order position with the given ID.
    ///
    /// Failed and incomplete check-ins are returned as a [`CheckinResult`], not as an
    /// error.
    pub fn redeem(&self, id: u64, body: &RedeemPosition) -> Request<CheckinResult> {
        self.client
            .post(&self.path.join(id).join("redeem"))
            .json_body(body)
            .decode_with(decode_checkin_result)
    }

    /// Tries to check in the order position with the given ticket secret
    /// (`untrusted_input=true`, so the secret is never interpreted as an ID).
    ///
    /// Failed and incomplete check-ins are returned as a [`CheckinResult`], not as an
    /// error.
    pub fn redeem_secret(&self, secret: &str, body: &RedeemPosition) -> Request<CheckinResult> {
        self.client
            .post(&self.path.join(secret).join("redeem"))
            .query_pair("untrusted_input", "true")
            .json_body(body)
            .decode_with(decode_checkin_result)
    }
}
