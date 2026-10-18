//! Order position endpoints.

use std::collections::BTreeMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::{CreateAnswer, OrderPosition, OrderStatus, PrintLog, PrintType, QuotaCheck};
use crate::path::Path;
use crate::query::Ordering;
use crate::types::{Decimal, string_enum};
use crate::{Client, Download, EventScope, JobStatus, OrganizerScope, Page, Request};

string_enum! {
    /// Fields order positions can be sorted by.
    pub enum OrderPositionOrdering {
        /// By order code.
        OrderCode = "order__code",
        /// By order creation time.
        OrderDatetime = "order__datetime",
        /// By position number within the order.
        Positionid = "positionid",
        /// By attendee name.
        AttendeeName = "attendee_name",
        /// By order status.
        OrderStatus = "order__status",
    }
}

/// Query parameters for [`OrderPositions::list`] and [`OrganizerOrderPositions::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListOrderPositions {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order (default: order creation time, then position number).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<OrderPositionOrdering>>,
    /// Only positions of the order with this code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub order: Option<String>,
    /// Fuzzy search on attendee name, order code, invoice address name and the
    /// beginning of the secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub search: Option<String>,
    /// Only positions of orders linked to this customer (identifier).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub customer: Option<String>,
    /// Only positions of this item ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<u64>,
    /// Only positions of one of these item IDs.
    #[serde(
        rename = "item__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub item_in: Vec<u64>,
    /// Only positions of this variation ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// Only positions of one of these variation IDs.
    #[serde(
        rename = "variation__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub variation_in: Vec<u64>,
    /// Only positions with this attendee name (add-ons of such positions included).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub attendee_name: Option<String>,
    /// Only positions with this ticket secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub secret: Option<String>,
    /// Only positions with this pseudonymization ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub pseudonymization_id: Option<String>,
    /// Only positions of orders in this status.
    #[serde(rename = "order__status", skip_serializing_if = "Option::is_none")]
    pub order_status: Option<OrderStatus>,
    /// Only positions of orders in one of these statuses.
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
    /// Only positions of this sub-event ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// Only positions of one of these sub-event IDs.
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
    /// Only add-ons to one of the positions with these IDs.
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
    /// Only positions with the voucher with this code.
    #[serde(rename = "voucher__code", skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub voucher_code: Option<String>,
    /// Include canceled positions (of orders that are not fully canceled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_canceled_positions: Option<bool>,
    /// Include `pdf_data`. Not supported by [`OrganizerOrderPositions::list`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_data: Option<bool>,
}

/// Query parameters for [`OrderPositions::get_with`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct GetOrderPosition {
    /// Allow fetching canceled positions (otherwise `404`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_canceled_positions: Option<bool>,
    /// Include `pdf_data`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_data: Option<bool>,
}

/// Body of [`OrderPositions::create`]: adds a position to an existing order.
///
/// Set either `attendee_name` or `attendee_name_parts`. No invoice is created
/// automatically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateOrderPosition {
    /// Code of the order.
    #[builder(into)]
    pub order: String,
    /// `positionid` of the parent position, for add-ons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addon_to: Option<u32>,
    /// ID of the item.
    pub item: u64,
    /// ID of the variation (required depending on the item).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// ID of the sub-event (required in event series).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// The `seat_guid` of the seat (required depending on event and item).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub seat: Option<String>,
    /// Price. The default price is used if unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Decimal>,
    /// Attendee email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub attendee_email: Option<String>,
    /// Attendee name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub attendee_name: Option<String>,
    /// Attendee name decomposition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attendee_name_parts: Option<BTreeMap<String, String>>,
    /// Attendee company name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub company: Option<String>,
    /// Attendee street.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub street: Option<String>,
    /// Attendee ZIP code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub zipcode: Option<String>,
    /// Attendee city.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub city: Option<String>,
    /// Attendee country code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub country: Option<String>,
    /// Attendee state (ISO 3166-2 code).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub state: Option<String>,
    /// Answers to questions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub answers: Vec<CreateAnswer>,
    /// Start of validity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<Timestamp>,
    /// End of validity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<Timestamp>,
}

/// Body of [`OrderPositions::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value. Changes do not create a new
/// invoice automatically, and a new `secret` is neither sent to the customer nor
/// added to the revocation list.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateOrderPosition {
    /// Attendee email address.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub attendee_email: Option<Option<String>>,
    /// Attendee name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub attendee_name: Option<String>,
    /// Attendee name decomposition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attendee_name_parts: Option<BTreeMap<String, String>>,
    /// Attendee company name.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub company: Option<Option<String>>,
    /// Attendee street.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub street: Option<Option<String>>,
    /// Attendee ZIP code.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub zipcode: Option<Option<String>>,
    /// Attendee city.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub city: Option<Option<String>>,
    /// Attendee country code.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub country: Option<Option<String>>,
    /// Attendee state (ISO 3166-2 code).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub state: Option<Option<String>>,
    /// **All** answers of the position (`file:keep` keeps an uploaded file).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answers: Option<Vec<CreateAnswer>>,
    /// ID of the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<u64>,
    /// ID of the variation.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub variation: Option<Option<u64>>,
    /// ID of the sub-event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// The `seat_guid` of the seat.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub seat: Option<Option<String>>,
    /// Price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Decimal>,
    /// ID of the tax rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_rule: Option<u64>,
    /// Start of validity.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub valid_from: Option<Option<Timestamp>>,
    /// End of validity.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub valid_until: Option<Option<Timestamp>>,
    /// Ticket secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub secret: Option<String>,
}

/// Body of [`OrderPositions::printlog`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreatePrintLog {
    /// Time of printing.
    pub datetime: Timestamp,
    /// Source of the print job, e.g. the name of the app used.
    #[builder(into)]
    pub source: String,
    /// Type of print.
    pub r#type: PrintType,
    /// Whether the print job (probably) resulted in a print.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub successful: Option<bool>,
    /// Additional data with client-dependent structure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Serialize)]
struct BlockName<'a> {
    name: &'a str,
}

/// Endpoints for the order positions of an event. Created with
/// [`EventScope::order_positions`].
#[derive(Debug, Clone)]
pub struct OrderPositions {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Order positions of this event.
    #[must_use]
    pub fn order_positions(&self) -> OrderPositions {
        OrderPositions {
            client: self.client(),
            path: self.path().join("orderpositions"),
        }
    }
}

impl OrderPositions {
    /// Lists order positions.
    pub fn list(&self, params: &ListOrderPositions) -> Request<Page<OrderPosition>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one order position.
    pub fn get(&self, id: u64) -> Request<OrderPosition> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Fetches one order position with additional query parameters.
    pub fn get_with(&self, id: u64, params: &GetOrderPosition) -> Request<OrderPosition> {
        self.client.get(&self.path.join(id)).query(params).json()
    }

    /// Adds a position to an existing order.
    pub fn create(&self, body: &CreateOrderPosition) -> Request<OrderPosition> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Like [`OrderPositions::create`], with control over the quota check.
    pub fn create_with(
        &self,
        body: &CreateOrderPosition,
        params: &QuotaCheck,
    ) -> Request<OrderPosition> {
        self.client
            .post(&self.path)
            .query(params)
            .json_body(body)
            .json()
    }

    /// Updates an order position.
    pub fn update(&self, id: u64, body: &UpdateOrderPosition) -> Request<OrderPosition> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Like [`OrderPositions::update`], with control over the quota check.
    pub fn update_with(
        &self,
        id: u64,
        body: &UpdateOrderPosition,
        params: &QuotaCheck,
    ) -> Request<OrderPosition> {
        self.client
            .patch(&self.path.join(id))
            .query(params)
            .json_body(body)
            .json()
    }

    /// Cancels an order position.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }

    /// Downloads the ticket of one position with the given output provider (e.g. `pdf`).
    ///
    /// Returns [`JobStatus::Pending`] while the file is being generated. The result may
    /// also be a `text/uri-list` response with a URL the user should open.
    pub fn download(&self, id: u64, output: &str) -> Request<JobStatus<Download>> {
        self.client
            .get(&self.path.join(id).join("download").join(output))
            .download_job()
    }

    /// Generates a new `secret` and `web_secret` for the position.
    pub fn regenerate_secrets(&self, id: u64) -> Request<OrderPosition> {
        self.client
            .post(&self.path.join(id).join("regenerate_secrets"))
            .json()
    }

    /// Blocks the position from being used. `name` is `admin` or starts with `api:`.
    pub fn add_block(&self, id: u64, name: &str) -> Request<OrderPosition> {
        self.client
            .post(&self.path.join(id).join("add_block"))
            .json_body(&BlockName { name })
            .json()
    }

    /// Removes a block. `name` is `admin` or starts with `api:`.
    pub fn remove_block(&self, id: u64, name: &str) -> Request<OrderPosition> {
        self.client
            .post(&self.path.join(id).join("remove_block"))
            .json_body(&BlockName { name })
            .json()
    }

    /// Records that the ticket has been printed.
    pub fn printlog(&self, id: u64, body: &CreatePrintLog) -> Request<PrintLog> {
        self.client
            .post(&self.path.join(id).join("printlog"))
            .json_body(body)
            .json()
    }
}

/// Organizer-wide list of order positions across all events. Created with
/// [`OrganizerScope::order_positions`].
#[derive(Debug, Clone)]
pub struct OrganizerOrderPositions {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Order positions of all events of this organizer.
    #[must_use]
    pub fn order_positions(&self) -> OrganizerOrderPositions {
        OrganizerOrderPositions {
            client: self.client(),
            path: self.path().join("orderpositions"),
        }
    }
}

impl OrganizerOrderPositions {
    /// Lists order positions of all events (with sufficient permissions). Results
    /// include `event`; `pdf_data` is not supported.
    pub fn list(&self, params: &ListOrderPositions) -> Request<Page<OrderPosition>> {
        self.client.get(&self.path).query(params).page()
    }
}
