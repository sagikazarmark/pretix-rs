//! Body of the order creation endpoint.

use std::collections::BTreeMap;

use jiff::Timestamp;
use jiff::civil::Date;
use serde::{Deserialize, Serialize};

use super::{CreateAnswer, CreateInvoiceAddress, FeeType, OrderStatus, TaxRoundingMode};
use crate::types::Decimal;

/// Body of [`Orders::create`](super::Orders::create).
///
/// This endpoint is meant for importing orders: it does not validate sales periods,
/// sales channels, add-on rules or prices, and does not calculate fees. See the pretix
/// documentation before using it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateOrder {
    /// Order code (only `A-Z` and `0-9`, without `O` and `1`). Generated if unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub code: Option<String>,
    /// `Pending` or `Paid`. Defaults to pending for non-free orders. Paid orders do not
    /// trigger emails or the `order_paid` signal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OrderStatus>,
    /// Customer identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub customer: Option<String>,
    /// Whether to create a test mode order (default `false`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testmode: Option<bool>,
    /// Cart IDs whose positions are deleted (and their quota credited) on success.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub consume_carts: Vec<String>,
    /// The customer email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub email: Option<String>,
    /// The customer phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub phone: Option<String>,
    /// The locale used for communication with the customer. Defaults to the event's
    /// default locale.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub locale: Option<String>,
    /// Sales channel identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub sales_channel: Option<String>,
    /// Identifier of an existing payment provider. Required for paid orders with a
    /// non-zero total (`manual` is recommended); use `free` for free orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub payment_provider: Option<String>,
    /// Internal `info` value of the created payment. Informational only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_info: Option<serde_json::Value>,
    /// Completion time of the payment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_date: Option<Timestamp>,
    /// Tax rounding mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_rounding_mode: Option<TaxRoundingMode>,
    /// Internal comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub comment: Option<String>,
    /// Internal date for a custom follow-up action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_followup_at: Option<Date>,
    /// Whether the check-in app should show a warning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkin_attention: Option<bool>,
    /// Text shown when a ticket of this order is scanned.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub checkin_text: Option<String>,
    /// Meta data for this order, only available through the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_meta: Option<serde_json::Map<String, serde_json::Value>>,
    /// Whether the order needs approval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<bool>,
    /// Whether a pending order is treated like a paid order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_if_pending: Option<bool>,
    /// Payment deadline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<Timestamp>,
    /// Invoice address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_address: Option<CreateInvoiceAddress>,
    /// Order positions. Add-ons must directly follow their parent position.
    pub positions: Vec<NewOrderPosition>,
    /// Fees.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub fees: Vec<NewOrderFee>,
    /// Ignore quotas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    /// Whether to send the regular order emails. `Some(None)` lets the sales channel
    /// settings decide; unset means `false`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub send_email: Option<Option<bool>>,
    /// Secrets of gift cards to pay the (pending) order with, in sequence. Cannot be
    /// combined with `payment_provider` and `payment_info`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub use_gift_cards: Vec<String>,
    /// Only validate the order and return the resulting order (code `PREVIEW`)
    /// without creating it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simulate: Option<bool>,
}

/// A position in [`CreateOrder`].
///
/// Set either `attendee_name` or `attendee_name_parts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct NewOrderPosition {
    /// Number of the position within the order. Required when using `addon_to`:
    /// incrementing integers starting at 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positionid: Option<u32>,
    /// ID of the item.
    pub item: u64,
    /// ID of the variation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// Price. Computed from the item if unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Decimal>,
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
    /// Code of a valid voucher.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub voucher: Option<String>,
    /// Attendee email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub attendee_email: Option<String>,
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
    /// Ticket secret. Generated if unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub secret: Option<String>,
    /// `positionid` of the parent position, for add-ons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addon_to: Option<u32>,
    /// ID of the sub-event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// Start of validity. If both `valid_from` and `valid_until` are unset, validity is
    /// computed from the item; `Some(None)` sends an explicit `null`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub valid_from: Option<Option<Timestamp>>,
    /// End of validity, see `valid_from`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub valid_until: Option<Option<Timestamp>>,
    /// Requested start of validity, instead of `valid_from`/`valid_until`. May or may
    /// not be respected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_valid_from: Option<Timestamp>,
    /// ID of a reusable medium to connect the ticket to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_reusable_medium: Option<u64>,
    /// ID of a discount. Only possible with `price`; disables automatic discounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount: Option<u64>,
    /// Answers to questions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub answers: Vec<CreateAnswer>,
}

/// A fee in [`CreateOrder`] or in [`ChangeOrder::create_fees`](super::ChangeOrder).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct NewOrderFee {
    /// Type of fee.
    pub fee_type: FeeType,
    /// Fee amount (or a percentage, see `treat_value_as_percentage`).
    pub value: Decimal,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub description: Option<String>,
    /// Internal type, e.g. the ID of the payment provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub internal_type: Option<String>,
    /// ID of the tax rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_rule: Option<u64>,
    /// Treat `value` as a percentage of the sum of all product prices (order creation
    /// only).
    #[serde(
        rename = "_treat_value_as_percentage",
        skip_serializing_if = "Option::is_none"
    )]
    pub treat_value_as_percentage: Option<bool>,
    /// Ignore `tax_rule` and tax the fee like the products (order creation only).
    #[serde(
        rename = "_split_taxes_like_products",
        skip_serializing_if = "Option::is_none"
    )]
    pub split_taxes_like_products: Option<bool>,
}
