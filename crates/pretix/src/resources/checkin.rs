//! Check-in: the ticket scanning API (`checkinrpc`) and the check-in history.
//!
//! <https://docs.pretix.eu/dev/api/resources/checkin.html>

use std::collections::BTreeMap;

use http::StatusCode;
use jiff::Timestamp;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::{Expand, Ordering};
use crate::request::RawResponse;
use crate::resources::items::MediaPolicy;
use crate::resources::orders::{OrderPosition, OrderStatus};
use crate::resources::questions::Question;
use crate::resources::reusable_media::MediumType;
use crate::types::string_enum;
use crate::{Client, EventScope, OrganizerScope, Page, Request, Result};

/// The result of a check-in attempt (redeeming a ticket).
///
/// pretix answers failed and incomplete check-ins with `400`/`404` and a body
/// describing why; these are returned as a `CheckinResult` instead of an error.
/// Check [`CheckinResult::status`] (and [`CheckinResult::reason`]) to find out whether
/// the attendee may enter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CheckinResult<P = CheckinPosition> {
    /// Outcome of the check-in.
    pub status: CheckinStatus,
    /// Reason code, only set on status `error`.
    #[serde(default)]
    pub reason: Option<CheckinErrorReason>,
    /// Human-readable explanation, only set on status `error` with reason `rules` or
    /// `invalid_time` (and may be `None` even then).
    #[serde(default)]
    pub reason_explanation: Option<String>,
    /// Error message of the `404` response for unknown tickets.
    #[serde(default)]
    pub detail: Option<String>,
    /// The matching order position, if any was found.
    ///
    /// `P` is `serde_json::Value` for requests with `expand`, which replaces IDs with
    /// objects.
    #[serde(default = "Option::default")]
    pub position: Option<P>,
    /// Whether the `checkin_attention` flag is set on the item or order.
    #[serde(default)]
    pub require_attention: bool,
    /// Additional texts to show to the user.
    #[serde(default)]
    pub checkin_texts: Vec<String>,
    /// Excerpt of the matching check-in list, if any was found.
    #[serde(default)]
    pub list: Option<CheckinListExcerpt>,
    /// Questions to be answered for check-in, only set on status `incomplete`.
    #[serde(default)]
    pub questions: Vec<Question>,
    /// Reusable media policy of the item, only set on status `exchange`.
    #[serde(default)]
    pub media_policy: Option<MediaPolicy>,
    /// Reusable media type of the item, only set on status `exchange`.
    #[serde(default)]
    pub media_type: Option<MediumType>,
}

/// An order position as returned by check-in endpoints.
///
/// The `checkins` of the position only include check-ins of the selected list, and
/// details of the order are included for convenience.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CheckinPosition {
    /// The order position.
    #[serde(flatten)]
    pub position: OrderPosition,
    /// Whether the `checkin_attention` flag is set on the item or order.
    #[serde(default)]
    pub require_attention: Option<bool>,
    /// Status of the order.
    #[serde(rename = "order__status", default)]
    pub order_status: Option<OrderStatus>,
    /// Whether the order is valid while pending.
    #[serde(rename = "order__valid_if_pending", default)]
    pub order_valid_if_pending: Option<bool>,
    /// Whether the order requires approval.
    #[serde(rename = "order__require_approval", default)]
    pub order_require_approval: Option<bool>,
    /// Locale of the order.
    #[serde(rename = "order__locale", default)]
    pub order_locale: Option<String>,
}

/// Excerpt of a check-in list, included in a [`CheckinResult`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CheckinListExcerpt {
    /// Internal ID of the check-in list.
    pub id: u64,
    /// Name of the check-in list.
    pub name: String,
    /// Slug of the event the list belongs to.
    pub event: String,
    /// ID of the event series date the list belongs to.
    pub subevent: Option<u64>,
    /// Whether the list contains tickets from pending orders.
    pub include_pending: bool,
}

/// The result of [`CheckinRpc::annul`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnnulmentResult {
    /// Always `ok`.
    pub status: CheckinStatus,
}

/// A check-in (successful or not), as recorded in the check-in history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Checkin {
    /// Internal ID of the check-in.
    pub id: u64,
    /// Whether the check-in was successful.
    pub successful: bool,
    /// Why the check-in was unsuccessful.
    pub error_reason: Option<CheckinErrorReason>,
    /// Additional, human-readable reason for the check-in to be unsuccessful.
    pub error_explanation: Option<String>,
    /// Internal ID of the order position (`None` for unknown scans).
    pub position: Option<u64>,
    /// Logical time when the check-in happened.
    pub datetime: Timestamp,
    /// Time when the check-in appeared on the server.
    pub created: Timestamp,
    /// Internal ID of the check-in list.
    pub list: u64,
    /// Whether the check-in was performed by the system automatically.
    pub auto_checked_in: bool,
    /// Internal ID of the gate.
    pub gate: Option<u64>,
    /// Internal ID of the device.
    pub device: Option<u64>,
    /// Organizer-internal ID of the device.
    pub device_id: Option<u64>,
    /// Type of check-in.
    pub r#type: CheckinType,
}

string_enum! {
    /// Outcome of a check-in attempt.
    pub enum CheckinStatus {
        /// The check-in succeeded.
        Ok = "ok",
        /// Questions need to be answered first.
        Incomplete = "incomplete",
        /// The ticket must be exchanged for a reusable medium.
        Exchange = "exchange",
        /// The check-in failed, see the reason.
        Error = "error",
    }
}

string_enum! {
    /// Why a check-in failed.
    pub enum CheckinErrorReason {
        /// Ticket is not known.
        Invalid = "invalid",
        /// Ticket is not paid for.
        Unpaid = "unpaid",
        /// Ticket has been blocked.
        Blocked = "blocked",
        /// Ticket is not valid at this time.
        InvalidTime = "invalid_time",
        /// Ticket is canceled or expired.
        Canceled = "canceled",
        /// Ticket already has been redeemed.
        AlreadyRedeemed = "already_redeemed",
        /// Tickets with this product may not be scanned at this device.
        Product = "product",
        /// Check-in prevented by a user-defined rule.
        Rules = "rules",
        /// Multiple tickets match the scan.
        Ambiguous = "ambiguous",
        /// Ticket code has been revoked.
        Revoked = "revoked",
        /// Order has not yet been approved.
        Unapproved = "unapproved",
        /// Ticket has been exchanged for a reusable medium that must be used instead.
        AlreadyExchanged = "already_exchanged",
        /// Reusable medium identifier was not found or is not valid.
        MediumInvalid = "medium_invalid",
        /// Reusable medium identifier already exists, but expected to be new.
        MediumExists = "medium_exists",
        /// Required questions were not answered (check-in history only).
        Incomplete = "incomplete",
        /// The check-in was annulled (check-in history only).
        Annulled = "annulled",
        /// Internal error.
        Error = "error",
    }
}

string_enum! {
    /// Direction of a check-in.
    pub enum CheckinType {
        /// Entry.
        Entry = "entry",
        /// Exit.
        Exit = "exit",
    }
}

string_enum! {
    /// Fields of order positions that can be expanded into full objects in check-in
    /// responses.
    pub enum CheckinPositionExpand {
        /// The sub-event.
        Subevent = "subevent",
        /// The item.
        Item = "item",
        /// The item variation.
        Variation = "variation",
        /// The questions of the answers.
        AnswersQuestion = "answers.question",
    }
}

string_enum! {
    /// Fields order positions can be sorted by in check-in searches.
    pub enum CheckinPositionOrdering {
        /// By order code.
        OrderCode = "order__code",
        /// By order date.
        OrderDatetime = "order__datetime",
        /// By position number within the order.
        Positionid = "positionid",
        /// By attendee name.
        AttendeeName = "attendee_name",
        /// By time of the last check-in.
        LastCheckedIn = "last_checked_in",
        /// By order email address.
        OrderEmail = "order__email",
    }
}

string_enum! {
    /// Fields check-ins can be sorted by.
    pub enum CheckinOrdering {
        /// By logical check-in time.
        Datetime = "datetime",
        /// By creation time on the server.
        Created = "created",
        /// By ID.
        Id = "id",
    }
}

/// Query parameters for [`CheckinRpc::search`] and [`CheckinRpc::search_with`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct SearchTickets {
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
    /// Check-in lists to search on (at most one per event).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub list: Vec<u64>,
    /// Fuzzy search matching the attendee name, order code, invoice address name and
    /// the beginning of the secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub search: Option<String>,
    /// Return results regardless of the order status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_status: Option<bool>,
    /// Only positions of the order with this code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub order: Option<String>,
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

/// Query parameters for [`Checkins::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct ListCheckins {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<CheckinOrdering>>,
    /// Only check-ins created at or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_since: Option<Timestamp>,
    /// Only check-ins created before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_before: Option<Timestamp>,
    /// Only check-ins that happened at or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime_since: Option<Timestamp>,
    /// Only check-ins that happened before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime_before: Option<Timestamp>,
    /// Only (un)successful check-ins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub successful: Option<bool>,
    /// Only check-ins with this error reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<CheckinErrorReason>,
    /// Only check-ins on this list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<u64>,
    /// Only check-ins of this type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<CheckinType>,
    /// Only check-ins from this gate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate: Option<u64>,
    /// Only check-ins from this device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<u64>,
    /// Only check-ins that are (not) automatic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_checked_in: Option<bool>,
}

/// Body of [`CheckinRpc::redeem`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct RedeemTicket {
    /// The scanned code (the ticket's `secret`). Safe to pass untrusted input.
    #[builder(into)]
    pub secret: String,
    /// Type of source the secret was obtained from. Defaults to `barcode`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<MediumType>,
    /// Check-in lists to search on (at most one per event).
    pub lists: Vec<u64>,
    /// Entry (default) or exit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<CheckinType>,
    /// Time of the check-in. Defaults to now.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime: Option<Timestamp>,
    /// Succeed regardless of revoked codes, previous check-ins or missing answers
    /// (e.g. to upload offline scans).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    /// Whether the client can handle questions. **Must** be `false` if not.
    /// Defaults to `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub questions_supported: Option<bool>,
    /// Succeed even if the order is pending (requires `include_pending` on the list).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_unpaid: Option<bool>,
    /// Answers by question ID. Choice answers are comma-separated option IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answers: Option<BTreeMap<u64, String>>,
    /// Unique random value identifying this check-in, to make retries idempotent.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub nonce: Option<String>,
    /// Type of the new reusable medium to exchange the ticket for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_medium_type: Option<MediumType>,
    /// Identifier of the new reusable medium to exchange the ticket for.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub exchange_medium_identifier: Option<String>,
    /// Use the order's locale for texts such as `reason_explanation`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_order_locale: Option<bool>,
    /// Only simulate the check-in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simulate: Option<bool>,
}

/// Body of [`CheckinRpc::annul`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct AnnulCheckin {
    /// `nonce` of the original check-in.
    #[builder(into)]
    pub nonce: String,
    /// Check-in lists to search on (at most one per event).
    pub lists: Vec<u64>,
    /// Client-side time of the annulment. Defaults to now.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime: Option<Timestamp>,
    /// Human-readable description of why the check-in was annulled.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub error_explanation: Option<String>,
}

/// Decodes the response of a redeem endpoint.
///
/// `400` and `404` responses with a `status` field are domain results (incomplete,
/// unknown ticket, …), not errors.
pub(crate) fn decode_checkin_result<P: DeserializeOwned>(
    raw: RawResponse,
) -> Result<CheckinResult<P>> {
    if raw.status.is_success() {
        return raw.json();
    }
    if !matches!(raw.status, StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND) {
        return Err(raw.into_error());
    }
    match raw.json::<CheckinResult<P>>() {
        Ok(result) => Ok(result),
        Err(_) => Err(raw.into_error()),
    }
}

/// Endpoints for scanning apps: redeeming, searching and annulling across events.
/// Created with [`OrganizerScope::checkin_rpc`].
#[derive(Debug, Clone)]
pub struct CheckinRpc {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Ticket scanning (check-in) endpoints of this organizer.
    #[must_use]
    pub fn checkin_rpc(&self) -> CheckinRpc {
        CheckinRpc {
            client: self.client(),
            path: self.path().join("checkinrpc"),
        }
    }
}

impl CheckinRpc {
    /// Tries to check in (or out) the ticket with the scanned secret.
    ///
    /// Failed and incomplete check-ins are returned as a [`CheckinResult`], not as an
    /// error.
    pub fn redeem(&self, body: &RedeemTicket) -> Request<CheckinResult> {
        self.client
            .post(&self.path.join("redeem"))
            .json_body(body)
            .decode_with(decode_checkin_result)
    }

    /// Like [`CheckinRpc::redeem`], expanding fields of the returned position (which
    /// is therefore returned as JSON).
    pub fn redeem_with(
        &self,
        body: &RedeemTicket,
        params: &Expand<CheckinPositionExpand>,
    ) -> Request<CheckinResult<serde_json::Value>> {
        self.client
            .post(&self.path.join("redeem"))
            .query(params)
            .json_body(body)
            .decode_with(decode_checkin_result)
    }

    /// Searches order positions on one or more check-in lists.
    pub fn search(&self, params: &SearchTickets) -> Request<Page<CheckinPosition>> {
        self.client
            .get(&self.path.join("search"))
            .query(params)
            .page()
    }

    /// Like [`CheckinRpc::search`], expanding fields of the positions (which are
    /// therefore returned as JSON).
    pub fn search_with(
        &self,
        params: &SearchTickets,
        expand: &Expand<CheckinPositionExpand>,
    ) -> Request<Page<serde_json::Value>> {
        self.client
            .get(&self.path.join("search"))
            .query(params)
            .query(expand)
            .page()
    }

    /// Annuls a check-in that was made in error (e.g. the turnstile did not turn).
    pub fn annul(&self, body: &AnnulCheckin) -> Request<AnnulmentResult> {
        self.client
            .post(&self.path.join("annul"))
            .json_body(body)
            .json()
    }
}

/// Endpoints for the check-in history of an event. Created with [`EventScope::checkins`].
#[derive(Debug, Clone)]
pub struct Checkins {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Check-in history of this event.
    #[must_use]
    pub fn checkins(&self) -> Checkins {
        Checkins {
            client: self.client(),
            path: self.path().join("checkins"),
        }
    }
}

impl Checkins {
    /// Lists check-ins (successful and failed).
    pub fn list(&self, params: &ListCheckins) -> Request<Page<Checkin>> {
        self.client.get(&self.path).query(params).page()
    }
}
