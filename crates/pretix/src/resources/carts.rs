//! Cart positions: reserve quota through the API.
//!
//! <https://docs.pretix.eu/dev/api/resources/carts.html>

use std::collections::BTreeMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::resources::orders::{Answer, AssignedSeat, CreateAnswer};
use crate::types::Decimal;
use crate::{Client, EventScope, ListParams, Page, Request};

/// A cart position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CartPosition {
    /// Internal ID of the cart position.
    pub id: u64,
    /// Identifier of the cart. Ends in `@api` for API-created positions.
    pub cart_id: String,
    /// Time of creation.
    pub datetime: Timestamp,
    /// The position expires and no longer blocks quota at this time.
    pub expires: Timestamp,
    /// ID of the item.
    pub item: u64,
    /// ID of the variation.
    #[serde(default)]
    pub variation: Option<u64>,
    /// Price of this position.
    pub price: Decimal,
    /// Attendee name.
    #[serde(default)]
    pub attendee_name: Option<String>,
    /// Decomposition of the attendee name.
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub attendee_name_parts: BTreeMap<String, String>,
    /// Attendee email address.
    #[serde(default)]
    pub attendee_email: Option<String>,
    /// Internal ID of the voucher used.
    #[serde(default)]
    pub voucher: Option<u64>,
    /// Internal ID of the position this position is an add-on for.
    #[serde(default)]
    pub addon_to: Option<u64>,
    /// If `addon_to` is set: whether this is a bundled (`true`) or add-on product.
    #[serde(default)]
    pub is_bundled: bool,
    /// ID of the event series date.
    #[serde(default)]
    pub subevent: Option<u64>,
    /// **Deprecated**: whether the price includes taxes.
    #[serde(default)]
    pub includes_tax: Option<bool>,
    /// Answers to user-defined questions.
    #[serde(default)]
    pub answers: Vec<Answer>,
    /// The assigned seat.
    #[serde(default)]
    pub seat: Option<AssignedSeat>,
    /// Nested add-on positions. Only included in creation responses.
    #[serde(default)]
    pub addons: Vec<CartPosition>,
    /// Nested bundled positions. Only included in creation responses.
    #[serde(default)]
    pub bundled: Vec<CartPosition>,
}

/// The result for one position of [`CartPositions::bulk_create`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BulkCreateResult {
    /// Whether this position was created.
    #[serde(deserialize_with = "crate::serde_util::lenient_bool")]
    pub success: bool,
    /// Validation errors, e.g. `{"non_field_errors": ["…"]}`.
    #[serde(default)]
    pub errors: Option<serde_json::Value>,
    /// The created position.
    #[serde(default)]
    pub data: Option<CartPosition>,
}

#[derive(Deserialize)]
struct BulkCreateResponse {
    results: Vec<BulkCreateResult>,
}

/// Body of [`CartPositions::create`] and [`CartPositions::bulk_create`].
///
/// **Experimental** in pretix. Prices and many restrictions are not validated. Set
/// either `attendee_name` or `attendee_name_parts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateCartPosition {
    /// Identifier of the cart; must end in `@api`. Generated if unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub cart_id: Option<String>,
    /// ID of the item.
    pub item: u64,
    /// ID of the variation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// Price.
    pub price: Decimal,
    /// The `seat_guid` of a seat. Required if the item requires a seat.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub seat: Option<String>,
    /// Attendee name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub attendee_name: Option<String>,
    /// Attendee name decomposition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attendee_name_parts: Option<BTreeMap<String, String>>,
    /// Attendee email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub attendee_email: Option<String>,
    /// ID of the event series date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// Expiry of the reservation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<Timestamp>,
    /// Sales channel identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub sales_channel: Option<String>,
    /// Voucher code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub voucher: Option<String>,
    /// Nested add-on positions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub addons: Vec<CreateCartAddon>,
    /// Nested bundled positions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub bundled: Vec<CreateCartAddon>,
    /// Answers to questions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub answers: Vec<CreateAnswer>,
}

/// A nested add-on or bundled position in [`CreateCartPosition`].
///
/// Set either `attendee_name` or `attendee_name_parts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateCartAddon {
    /// ID of the item.
    pub item: u64,
    /// ID of the variation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// Price.
    pub price: Decimal,
    /// Attendee name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub attendee_name: Option<String>,
    /// Attendee name decomposition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attendee_name_parts: Option<BTreeMap<String, String>>,
    /// Attendee email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub attendee_email: Option<String>,
    /// ID of the event series date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// Answers to questions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub answers: Vec<CreateAnswer>,
}

/// Endpoints for the API-created cart positions of an event. Created with
/// [`EventScope::cart_positions`].
#[derive(Debug, Clone)]
pub struct CartPositions {
    client: Client,
    path: Path,
}

impl EventScope {
    /// API-created cart positions of this event.
    #[must_use]
    pub fn cart_positions(&self) -> CartPositions {
        CartPositions {
            client: self.client(),
            path: self.path().join("cartpositions"),
        }
    }
}

impl CartPositions {
    /// Lists API-created cart positions.
    pub fn list(&self, params: &ListParams) -> Request<Page<CartPosition>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one cart position.
    pub fn get(&self, id: u64) -> Request<CartPosition> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a cart position (including nested `addons` and `bundled` positions).
    pub fn create(&self, body: &CreateCartPosition) -> Request<CartPosition> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Creates multiple cart positions. **Not atomic**: check `success` of every
    /// result, which are in the order of `positions`.
    pub fn bulk_create(&self, positions: &[CreateCartPosition]) -> Request<Vec<BulkCreateResult>> {
        self.client
            .post(&self.path.join("bulk_create"))
            .json_body(positions)
            .decode_with(|raw| {
                raw.ensure_success()?
                    .json::<BulkCreateResponse>()
                    .map(|response| response.results)
            })
    }

    /// Deletes a cart position.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
