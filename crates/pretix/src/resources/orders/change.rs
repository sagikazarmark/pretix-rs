//! Body of the order change endpoint.

use std::collections::BTreeMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::{CreateAnswer, NewOrderFee};
use crate::types::{Decimal, string_enum};

string_enum! {
    /// How taxes are recalculated when changing an order.
    pub enum RecalculateTaxes {
        /// Recalculate taxes, keeping net prices.
        KeepNet = "keep_net",
        /// Recalculate taxes, keeping gross prices.
        KeepGross = "keep_gross",
    }
}

/// Body of [`Orders::change`](super::Orders::change): multiple changes performed in one
/// transaction.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ChangeOrder {
    /// Positions to change.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub patch_positions: Vec<PatchPosition>,
    /// Positions to cancel.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub cancel_positions: Vec<CancelPosition>,
    /// Positions to split into a new order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub split_positions: Vec<SplitPosition>,
    /// Positions to add.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub create_positions: Vec<AddPosition>,
    /// Fees to change.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub patch_fees: Vec<PatchFee>,
    /// Fees to cancel.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub cancel_fees: Vec<CancelFee>,
    /// Fees to add.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub create_fees: Vec<NewOrderFee>,
    /// Recalculate taxes based on tax rules and invoice address. Unset: no
    /// recalculation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recalculate_taxes: Option<RecalculateTaxes>,
    /// Notify the customer about the change (default `false`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_email: Option<bool>,
    /// Cancel and reissue an existing invoice (default `true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reissue_invoice: Option<bool>,
}

/// A position change in [`ChangeOrder`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct PatchPosition {
    /// ID of the order position.
    pub position: u64,
    /// The values to change.
    pub body: PositionChanges,
}

/// Changed values of a position in [`PatchPosition`]. Unset fields are left
/// unchanged; for nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct PositionChanges {
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
}

/// A position to cancel in [`ChangeOrder`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CancelPosition {
    /// ID of the order position.
    pub position: u64,
}

/// A position to split into a new order in [`ChangeOrder`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct SplitPosition {
    /// ID of the order position.
    pub position: u64,
}

/// A position to add in [`ChangeOrder`]. Supports the same fields as
/// [`CreateOrderPosition`](super::CreateOrderPosition), except `order`.
///
/// Set either `attendee_name` or `attendee_name_parts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct AddPosition {
    /// `positionid` of the parent position, for add-ons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addon_to: Option<u32>,
    /// ID of the item.
    pub item: u64,
    /// ID of the variation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// ID of the sub-event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// The `seat_guid` of the seat.
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

/// A fee change in [`ChangeOrder`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct PatchFee {
    /// ID of the fee.
    pub fee: u64,
    /// The values to change.
    pub body: FeeChanges,
}

/// Changed values of a fee in [`PatchFee`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct FeeChanges {
    /// Fee amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Decimal>,
}

/// A fee to cancel in [`ChangeOrder`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CancelFee {
    /// ID of the fee.
    pub fee: u64,
}
