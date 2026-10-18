//! Seats of the seating plan of an event or a sub-event.
//!
//! Events without sub-events use [`EventScope::seats`]; event series use
//! [`SubEvents::seats`] with the ID of a sub-event. Both return the same
//! [`Seats`] handle (pretix responds with `404` if the wrong one is used).
//!
//! <https://docs.pretix.eu/dev/api/resources/seats.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Expand;
use crate::resources::subevents::SubEvents;
use crate::resources::vouchers::Voucher;
use crate::types::{Expandable, string_enum};
use crate::{Client, EventScope, Page, Request};

/// A seat.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct Seat {
    /// Internal ID of the seat.
    pub id: u64,
    /// ID of the sub-event this seat belongs to.
    pub subevent: Option<u64>,
    /// Name of the zone the seat is in.
    pub zone_name: String,
    /// Name or number of the row the seat is in.
    pub row_name: String,
    /// Additional label of the row.
    pub row_label: Option<String>,
    /// Number of the seat within the row.
    pub seat_number: String,
    /// Additional label of the seat.
    pub seat_label: Option<String>,
    /// Identifier of the seat within the seating plan.
    pub seat_guid: String,
    /// ID of the product mapped to this seat.
    pub product: Option<u64>,
    /// Whether the seat is blocked manually.
    pub blocked: bool,
    /// The order position reserving this seat: its ID, or the nested order position if
    /// expanded with [`SeatExpansion::OrderPosition`].
    ///
    /// Expanded positions are returned as JSON: they differ from
    /// [`OrderPosition`](crate::resources::orders::OrderPosition) by an `order` object
    /// (`{"code": "ABCDE", "event": "eventslug"}`) and lack `seat`.
    pub orderposition: Option<Expandable<u64, serde_json::Value>>,
    /// The cart position reserving this seat: its ID, or the nested cart position if
    /// expanded with [`SeatExpansion::CartPosition`].
    pub cartposition: Option<Expandable<u64, serde_json::Value>>,
    /// The voucher reserving this seat: its ID, or the nested voucher if expanded with
    /// [`SeatExpansion::Voucher`].
    pub voucher: Option<Expandable<u64, Voucher>>,
}

string_enum! {
    /// Related objects that can be shown nested instead of as an ID.
    pub enum SeatExpansion {
        /// `orderposition`.
        OrderPosition = "orderposition",
        /// `cartposition`.
        CartPosition = "cartposition",
        /// `voucher`.
        Voucher = "voucher",
    }
}

/// Query parameters for [`Seats::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListSeats {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Only seats in this zone.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub zone_name: Option<String>,
    /// Only seats with this row name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub row_name: Option<String>,
    /// Only seats with this row label.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub row_label: Option<String>,
    /// Only seats with this seat number.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub seat_number: Option<String>,
    /// Only seats with this seat label.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub seat_label: Option<String>,
    /// Only the seat with this GUID.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub seat_guid: Option<String>,
    /// Only seats that are (`true`) or are not (`false`) blocked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked: Option<bool>,
    /// Only seats that are (`true`) or are not (`false`) currently available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_available: Option<bool>,
    /// Related objects to show nested instead of as an ID (requires permission to
    /// access them).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub expand: Vec<SeatExpansion>,
}

/// Body of [`Seats::update`]. Only `blocked` can be changed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateSeat {
    /// Whether the seat is blocked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked: Option<bool>,
}

/// Body of [`Seats::bulk_block`] and [`Seats::bulk_unblock`]: up to 10,000 seats,
/// given either by `ids` or by `seat_guids`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct BulkBlockSeats {
    /// IDs of the seats.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub ids: Vec<u64>,
    /// `seat_guid`s of the seats.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub seat_guids: Vec<String>,
}

/// Body of [`Seats::bulk_unblock`].
pub type BulkUnblockSeats = BulkBlockSeats;

/// Endpoints for the seats of an event or a sub-event. Created with
/// [`EventScope::seats`] or [`SubEvents::seats`].
#[derive(Debug, Clone)]
pub struct Seats {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Seats of this event. Only for events without sub-events.
    #[must_use]
    pub fn seats(&self) -> Seats {
        Seats {
            client: self.client(),
            path: self.path().join("seats"),
        }
    }
}

impl SubEvents {
    /// Seats of the sub-event `subevent_id` of this event series.
    #[must_use]
    pub fn seats(&self, subevent_id: u64) -> Seats {
        Seats {
            client: self.client(),
            path: self.path().join(subevent_id).join("seats"),
        }
    }
}

impl Seats {
    /// Lists seats.
    pub fn list(&self, params: &ListSeats) -> Request<Page<Seat>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one seat.
    pub fn get(&self, id: u64) -> Request<Seat> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Fetches one seat, showing the expanded related objects nested instead of as
    /// an ID.
    pub fn get_with(&self, id: u64, params: &Expand<SeatExpansion>) -> Request<Seat> {
        self.client.get(&self.path.join(id)).query(params).json()
    }

    /// Updates a seat.
    pub fn update(&self, id: u64, body: &UpdateSeat) -> Request<Seat> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Blocks many seats at once. Seats that are already blocked or sold are ignored.
    pub fn bulk_block(&self, body: &BulkBlockSeats) -> Request<()> {
        self.client
            .post(&self.path.join("bulk_block"))
            .json_body(body)
            .empty()
    }

    /// Unblocks many seats at once. Seats that are already unblocked or sold are
    /// ignored.
    pub fn bulk_unblock(&self, body: &BulkUnblockSeats) -> Request<()> {
        self.client
            .post(&self.path.join("bulk_unblock"))
            .json_body(body)
            .empty()
    }
}
