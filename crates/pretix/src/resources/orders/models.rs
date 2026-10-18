//! Response models shared by orders, order positions, payments and refunds.

use std::collections::BTreeMap;

use jiff::Timestamp;
use jiff::civil::Date;
use serde::{Deserialize, Serialize};

use crate::resources::checkin::CheckinType;
use crate::resources::invoices::TransmissionType;
use crate::types::{Decimal, string_enum};

/// An order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Order {
    /// Order code.
    pub code: String,
    /// The slug of the parent event.
    pub event: String,
    /// Order status.
    pub status: OrderStatus,
    /// Whether the order was created in test mode. Only test mode orders can be deleted.
    pub testmode: bool,
    /// The secret contained in the link sent to the customer.
    pub secret: String,
    /// The customer email address.
    pub email: Option<String>,
    /// The customer phone number.
    #[serde(default)]
    pub phone: Option<String>,
    /// The customer account identifier.
    #[serde(default)]
    pub customer: Option<String>,
    /// The locale used for communication with this customer.
    pub locale: Option<String>,
    /// Channel this sale was created through, such as `web`.
    pub sales_channel: String,
    /// Time of order creation.
    pub datetime: Timestamp,
    /// The order will expire at this time if it is still pending.
    pub expires: Timestamp,
    /// **Deprecated and inaccurate**: date of payment receipt.
    #[serde(default)]
    pub payment_date: Option<Date>,
    /// **Deprecated and inaccurate**: payment provider used for this order.
    #[serde(default)]
    pub payment_provider: Option<String>,
    /// Total value of this order.
    pub total: Decimal,
    /// Tax rounding mode.
    #[serde(default)]
    pub tax_rounding_mode: Option<TaxRoundingMode>,
    /// Internal comment on this order.
    #[serde(default)]
    pub comment: Option<String>,
    /// Meta data for this order, only available through the API. Not included in list
    /// responses.
    #[serde(default)]
    pub api_meta: serde_json::Map<String, serde_json::Value>,
    /// Internal date for a custom follow-up action.
    #[serde(default)]
    pub custom_followup_at: Option<Date>,
    /// Whether the check-in app should warn that this ticket requires special attention.
    pub checkin_attention: bool,
    /// Text shown when a ticket of this order is scanned.
    #[serde(default)]
    pub checkin_text: Option<String>,
    /// Invoice address information.
    #[serde(default)]
    pub invoice_address: Option<InvoiceAddress>,
    /// Order positions. By default, only non-canceled positions are included.
    #[serde(default)]
    pub positions: Vec<OrderPosition>,
    /// Fees included in the order total. By default, only non-canceled fees are included.
    #[serde(default)]
    pub fees: Vec<OrderFee>,
    /// Ticket download options for order-wise ticket downloading.
    #[serde(default)]
    pub downloads: Vec<TicketDownload>,
    /// If pending: the order needs approval. If canceled: the order has been denied.
    pub require_approval: bool,
    /// If pending: the order is still treated like a paid order for most purposes.
    #[serde(default)]
    pub valid_if_pending: bool,
    /// The full URL to the order confirmation page.
    pub url: String,
    /// Payment processes.
    #[serde(default)]
    pub payments: Vec<OrderPayment>,
    /// Refund processes.
    #[serde(default)]
    pub refunds: Vec<OrderRefund>,
    /// Last modification of this object.
    pub last_modified: Timestamp,
    /// Time of order cancellation. Not set for partial cancellations.
    #[serde(default)]
    pub cancellation_date: Option<Timestamp>,
    /// Additional data added by plugins.
    #[serde(default)]
    pub plugin_data: serde_json::Map<String, serde_json::Value>,
    /// Unique secrets of the gift cards used to pay for this order.
    #[serde(default)]
    pub use_gift_cards: Vec<String>,
}

/// An invoice address of an order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InvoiceAddress {
    /// Last modification date of the address.
    #[serde(default)]
    pub last_modified: Option<Timestamp>,
    /// Customer company name.
    #[serde(default)]
    pub company: Option<String>,
    /// Business (`true`) or individual customer. Always `false` for orders created
    /// before pretix 1.7.
    #[serde(default)]
    pub is_business: bool,
    /// Customer name.
    #[serde(default)]
    pub name: Option<String>,
    /// Customer name decomposition.
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub name_parts: BTreeMap<String, String>,
    /// Customer street.
    #[serde(default)]
    pub street: Option<String>,
    /// Customer ZIP code.
    #[serde(default)]
    pub zipcode: Option<String>,
    /// Customer city.
    #[serde(default)]
    pub city: Option<String>,
    /// Customer country code.
    #[serde(default)]
    pub country: Option<String>,
    /// Customer state (ISO 3166-2 code). Only supported in some countries.
    #[serde(default)]
    pub state: Option<String>,
    /// Customer's internal reference to be printed on the invoice.
    #[serde(default)]
    pub internal_reference: Option<String>,
    /// Custom invoice address field.
    #[serde(default)]
    pub custom_field: Option<String>,
    /// Customer VAT ID.
    #[serde(default)]
    pub vat_id: Option<String>,
    /// Whether the VAT ID has been validated successfully against the EU VAT service.
    #[serde(default)]
    pub vat_id_validated: bool,
    /// Transmission channel for invoices. Defaults to `email`.
    #[serde(default)]
    pub transmission_type: Option<TransmissionType>,
    /// Transmission-channel specific information.
    #[serde(default)]
    pub transmission_info: Option<serde_json::Map<String, serde_json::Value>>,
}

/// A fee included in an order total.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OrderFee {
    /// Internal ID of the fee record.
    pub id: u64,
    /// Type of fee.
    pub fee_type: FeeType,
    /// Fee amount.
    pub value: Decimal,
    /// Human-readable string with more details (can be empty).
    #[serde(default)]
    pub description: String,
    /// Internal string, e.g. the ID of the payment provider (can be empty).
    #[serde(default)]
    pub internal_type: String,
    /// VAT rate applied for this fee.
    pub tax_rate: Decimal,
    /// VAT included in this fee.
    pub tax_value: Decimal,
    /// The ID of the used tax rule.
    pub tax_rule: Option<u64>,
    /// Codified reason for the tax rate.
    #[serde(default)]
    pub tax_code: Option<String>,
    /// Whether this fee has been canceled.
    #[serde(default)]
    pub canceled: bool,
}

/// A ticket download option.
///
/// Fetch the file with [`Client::download`](crate::Client::download) or
/// [`Client::download_job`](crate::Client::download_job).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TicketDownload {
    /// Ticket output provider, e.g. `pdf` or `passbook`.
    pub output: String,
    /// Download URL.
    pub url: String,
}

/// An order position (a ticket or other purchased product).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OrderPosition {
    /// Internal ID of the order position.
    pub id: u64,
    /// Order code of the order the position belongs to.
    pub order: String,
    /// The slug of the event. Only included in organizer-level lists.
    #[serde(default)]
    pub event: Option<String>,
    /// Number of the position within the order.
    pub positionid: u32,
    /// Whether this position has been canceled.
    #[serde(default)]
    pub canceled: bool,
    /// ID of the purchased item.
    pub item: u64,
    /// ID of the purchased variation.
    #[serde(default)]
    pub variation: Option<u64>,
    /// Price of this position.
    pub price: Decimal,
    /// Attendee name.
    #[serde(default)]
    pub attendee_name: Option<String>,
    /// Decomposition of the attendee name (e.g. given name, family name).
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub attendee_name_parts: BTreeMap<String, String>,
    /// Attendee email address.
    #[serde(default)]
    pub attendee_email: Option<String>,
    /// Attendee company name.
    #[serde(default)]
    pub company: Option<String>,
    /// Attendee street.
    #[serde(default)]
    pub street: Option<String>,
    /// Attendee ZIP code.
    #[serde(default)]
    pub zipcode: Option<String>,
    /// Attendee city.
    #[serde(default)]
    pub city: Option<String>,
    /// Attendee country code.
    #[serde(default)]
    pub country: Option<String>,
    /// Attendee state (ISO 3166-2 code). Only supported in some countries.
    #[serde(default)]
    pub state: Option<String>,
    /// Internal ID of the voucher used for this position.
    #[serde(default)]
    pub voucher: Option<u64>,
    /// Amount discounted by the voucher (consumed voucher budget). Unreliable after
    /// changes to the position.
    #[serde(default)]
    pub voucher_budget_use: Option<Decimal>,
    /// VAT rate applied for this position.
    pub tax_rate: Decimal,
    /// VAT included in this position.
    pub tax_value: Decimal,
    /// Codified reason for the tax rate.
    #[serde(default)]
    pub tax_code: Option<String>,
    /// The ID of the used tax rule.
    #[serde(default)]
    pub tax_rule: Option<u64>,
    /// Secret code printed on the tickets for validation.
    pub secret: String,
    /// Internal ID of the position this position is an add-on for.
    #[serde(default)]
    pub addon_to: Option<u64>,
    /// ID of the date inside an event series this position belongs to.
    #[serde(default)]
    pub subevent: Option<u64>,
    /// ID of a discount that has been used during creation of this position.
    #[serde(default)]
    pub discount: Option<u64>,
    /// Reasons this ticket is blocked. If not `None`, the ticket may not be used.
    #[serde(default)]
    pub blocked: Option<Vec<String>>,
    /// The ticket is not valid before this time.
    #[serde(default)]
    pub valid_from: Option<Timestamp>,
    /// The ticket is not valid after this time.
    #[serde(default)]
    pub valid_until: Option<Timestamp>,
    /// A random ID, e.g. for use in lead scanning apps.
    #[serde(default)]
    pub pseudonymization_id: String,
    /// **Successful** check-ins with this ticket.
    #[serde(default)]
    pub checkins: Vec<PositionCheckin>,
    /// Print jobs recorded e.g. by the pretix apps.
    #[serde(default)]
    pub print_logs: Vec<PrintLog>,
    /// Ticket download options.
    #[serde(default)]
    pub downloads: Vec<TicketDownload>,
    /// Answers to user-defined questions.
    #[serde(default)]
    pub answers: Vec<Answer>,
    /// The assigned seat.
    #[serde(default)]
    pub seat: Option<AssignedSeat>,
    /// Data required for ticket PDF generation. Only included when requested with
    /// `pdf_data=true`.
    #[serde(default)]
    pub pdf_data: Option<serde_json::Map<String, serde_json::Value>>,
    /// Additional data added by plugins.
    #[serde(default)]
    pub plugin_data: serde_json::Map<String, serde_json::Value>,
}

/// A successful check-in of an order position, as included in [`OrderPosition::checkins`].
///
/// The check-in history ([`Checkins`](crate::resources::checkin::Checkins)) uses
/// [`checkin::Checkin`](crate::resources::checkin::Checkin) instead.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PositionCheckin {
    /// Internal ID of the check-in event.
    pub id: u64,
    /// Internal ID of the check-in list.
    pub list: u64,
    /// Time of check-in.
    pub datetime: Timestamp,
    /// Type of scan.
    pub r#type: CheckinType,
    /// Internal ID of the gate.
    #[serde(default)]
    pub gate: Option<u64>,
    /// **Deprecated**: internal ID of the device.
    #[serde(default)]
    pub device: Option<u64>,
    /// The `device_id` attribute of the device.
    #[serde(default)]
    pub device_id: Option<u64>,
    /// Whether this check-in was performed automatically by the system.
    #[serde(default)]
    pub auto_checked_in: bool,
}

/// A print job of an order position, e.g. recorded by the pretix apps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PrintLog {
    /// Internal ID of the print job.
    pub id: u64,
    /// Whether the print job (probably) resulted in a print.
    #[serde(default)]
    pub successful: Option<bool>,
    /// The `device_id` attribute of the device that recorded the print.
    #[serde(default)]
    pub device_id: Option<u64>,
    /// Time of printing.
    pub datetime: Timestamp,
    /// Source of the print job, e.g. the name of the app used.
    pub source: String,
    /// Type of print.
    pub r#type: PrintType,
    /// Additional data with client-dependent structure.
    #[serde(default)]
    pub info: serde_json::Map<String, serde_json::Value>,
}

/// An answer to a user-defined question.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Answer {
    /// Internal ID of the answered question.
    pub question: u64,
    /// Text representation of the answer (URL if the answer is a file).
    pub answer: String,
    /// The question's `identifier` field.
    #[serde(default)]
    pub question_identifier: Option<String>,
    /// Internal IDs of the selected options (only for choice questions).
    #[serde(default)]
    pub options: Vec<u64>,
    /// The `identifier` fields of the selected options.
    #[serde(default)]
    pub option_identifiers: Vec<String>,
}

/// A seat assigned to an order or cart position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AssignedSeat {
    /// Internal ID of the seat instance.
    pub id: u64,
    /// Human-readable seat name.
    pub name: String,
    /// Name of the zone the seat is in.
    #[serde(default)]
    pub zone_name: String,
    /// Name or number of the row the seat is in.
    #[serde(default)]
    pub row_name: String,
    /// Additional label of the row.
    #[serde(default)]
    pub row_label: Option<String>,
    /// Number of the seat within the row.
    #[serde(default)]
    pub seat_number: String,
    /// Additional label of the seat.
    #[serde(default)]
    pub seat_label: Option<String>,
    /// Identifier of the seat within the seating plan.
    pub seat_guid: String,
}

/// A payment process of an order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OrderPayment {
    /// Internal ID of this payment, starting at 1 for every order.
    pub local_id: u64,
    /// Payment state.
    pub state: PaymentState,
    /// Payment amount.
    pub amount: Decimal,
    /// Time of creation of this payment.
    pub created: Timestamp,
    /// Time of completion of this payment.
    #[serde(default)]
    pub payment_date: Option<Timestamp>,
    /// Identification string of the payment provider.
    pub provider: String,
    /// The URL where a user can continue with the payment.
    #[serde(default)]
    pub payment_url: Option<String>,
    /// Payment-specific information. Its content differs between payment providers,
    /// versions and states; expect values to be missing.
    #[serde(default)]
    pub details: serde_json::Map<String, serde_json::Value>,
}

/// A refund process of an order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OrderRefund {
    /// Internal ID of this refund, starting at 1 for every order.
    pub local_id: u64,
    /// Refund state.
    pub state: RefundState,
    /// How this refund has been created.
    pub source: RefundSource,
    /// Refund amount.
    pub amount: Decimal,
    /// `local_id` of the payment this refund belongs to.
    #[serde(default)]
    pub payment: Option<u64>,
    /// Time of creation of this refund.
    pub created: Timestamp,
    /// Reason for the refund (shown to the customer in some cases).
    #[serde(default)]
    pub comment: Option<String>,
    /// Time of completion of this refund.
    #[serde(default)]
    pub execution_date: Option<Timestamp>,
    /// Identification string of the payment provider.
    pub provider: String,
    /// Refund-specific information. Its content differs between payment providers,
    /// versions and states; expect values to be missing.
    #[serde(default)]
    pub details: serde_json::Map<String, serde_json::Value>,
}

string_enum! {
    /// The status of an order.
    pub enum OrderStatus {
        /// Pending payment.
        Pending = "n",
        /// Paid.
        Paid = "p",
        /// Expired.
        Expired = "e",
        /// Canceled.
        Canceled = "c",
    }
}

string_enum! {
    /// How taxes of an order are rounded.
    pub enum TaxRoundingMode {
        /// Taxes are rounded for every line individually.
        Line = "line",
        /// Taxes and gross total are computed from the sum of net prices.
        SumByNet = "sum_by_net",
        /// Taxes are computed from the sum of net prices, keeping gross prices stable.
        SumByNetKeepGross = "sum_by_net_keep_gross",
    }
}

string_enum! {
    /// The type of an order fee.
    pub enum FeeType {
        /// Payment fee.
        Payment = "payment",
        /// Shipping fee.
        Shipping = "shipping",
        /// Service fee.
        Service = "service",
        /// Cancellation fee.
        Cancellation = "cancellation",
        /// Insurance fee.
        Insurance = "insurance",
        /// Late fee.
        Late = "late",
        /// Other fee (`other`).
        Other = "other",
        /// Gift card.
        Giftcard = "giftcard",
    }
}

string_enum! {
    /// The type of a print job.
    pub enum PrintType {
        /// Badge.
        Badge = "badge",
        /// Ticket.
        Ticket = "ticket",
        /// Certificate.
        Certificate = "certificate",
        /// Something else (`other`).
        Other = "other",
    }
}

string_enum! {
    /// The state of a payment.
    pub enum PaymentState {
        /// Created.
        Created = "created",
        /// Pending.
        Pending = "pending",
        /// Confirmed.
        Confirmed = "confirmed",
        /// Canceled.
        Canceled = "canceled",
        /// Failed.
        Failed = "failed",
        /// Refunded.
        Refunded = "refunded",
    }
}

string_enum! {
    /// The state of a refund.
    pub enum RefundState {
        /// Created.
        Created = "created",
        /// In transit.
        Transit = "transit",
        /// Created externally, needs to be processed.
        External = "external",
        /// Canceled.
        Canceled = "canceled",
        /// Failed.
        Failed = "failed",
        /// Done.
        Done = "done",
    }
}

string_enum! {
    /// How a refund has been created.
    pub enum RefundSource {
        /// Requested by the buyer.
        Buyer = "buyer",
        /// Created by an admin.
        Admin = "admin",
        /// Created externally (e.g. a chargeback).
        External = "external",
    }
}
