//! Orders, order positions, payments, refunds and revoked/blocked ticket secrets.
//!
//! <https://docs.pretix.eu/dev/api/resources/orders.html>

mod change;
mod create;
mod models;
mod payments;
mod positions;
mod refunds;
mod secrets;

use std::collections::BTreeMap;

use jiff::Timestamp;
use jiff::civil::Date;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::{FieldSelection, Ordering};
use crate::resources::invoices::{Invoice, TransmissionType};
use crate::types::{Decimal, string_enum};
use crate::{Client, Download, EventScope, JobStatus, OrganizerScope, Page, Request};

pub use crate::resources::checkin::CheckinType;
pub use change::{
    AddPosition, CancelFee, CancelPosition, ChangeOrder, FeeChanges, PatchFee, PatchPosition,
    PositionChanges, RecalculateTaxes, SplitPosition,
};
pub use create::{CreateOrder, NewOrderFee, NewOrderPosition};
pub use models::{
    Answer, AssignedSeat, FeeType, InvoiceAddress, Order, OrderFee, OrderPayment, OrderPosition,
    OrderRefund, OrderStatus, PaymentState, PositionCheckin, PrintLog, PrintType, RefundSource,
    RefundState, TaxRoundingMode, TicketDownload,
};
pub use payments::{ConfirmPayment, CreateOrderPayment, OrderPayments, RefundPayment};
pub use positions::{
    CreateOrderPosition, CreatePrintLog, GetOrderPosition, ListOrderPositions,
    OrderPositionOrdering, OrderPositions, OrganizerOrderPositions, UpdateOrderPosition,
};
pub use refunds::{CreateOrderRefund, OrderRefunds, ProcessRefund};
pub use secrets::{
    BlockedSecret, BlockedSecrets, ListBlockedSecrets, ListRevokedSecrets, RevokedSecret,
    RevokedSecretOrdering, RevokedSecrets,
};

string_enum! {
    /// Fields orders can be sorted by.
    pub enum OrderOrdering {
        /// By creation time (default).
        Datetime = "datetime",
        /// By order code.
        Code = "code",
        /// By last modification.
        LastModified = "last_modified",
        /// By status.
        Status = "status",
        /// By cancellation date.
        CancellationDate = "cancellation_date",
    }
}

/// Query parameters for [`Orders::list`] and [`OrganizerOrders::list`].
///
/// To select response fields (`include`/`exclude`), use [`Orders::list_with`] or
/// [`OrganizerOrders::list_with`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListOrders {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<OrderOrdering>>,
    /// Only orders with this order code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub code: Option<String>,
    /// Only orders in this status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OrderStatus>,
    /// Only orders matching this search query (names, email addresses, company names).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub search: Option<String>,
    /// Only orders linked to this customer (identifier).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub customer: Option<String>,
    /// Only orders with a position (possibly canceled) of this item ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<u64>,
    /// Only orders with a position (possibly canceled) of this variation ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// Only orders with this `testmode` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testmode: Option<bool>,
    /// Only orders with this `require_approval` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<bool>,
    /// Include canceled positions (of orders that are not fully canceled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_canceled_positions: Option<bool>,
    /// Include canceled fees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_canceled_fees: Option<bool>,
    /// Only orders created with this email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub email: Option<String>,
    /// Only orders with this customer locale.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub locale: Option<String>,
    /// Only orders changed since this time. Use the
    /// [`page_generated`](crate::ResponseMeta::page_generated) time of a previous list
    /// response, preferably together with `testmode=false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_since: Option<Timestamp>,
    /// Only orders created at or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_since: Option<Timestamp>,
    /// Only orders created before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_before: Option<Timestamp>,
    /// Only orders with a position (possibly canceled) for this sub-event ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// Only orders with a ticket for a sub-event ending after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent_after: Option<Timestamp>,
    /// Only orders with a ticket for a sub-event starting before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent_before: Option<Timestamp>,
    /// Only orders with this sales channel identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub sales_channel: Option<String>,
    /// Only orders with a payment (in any state) using this payment provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub payment_provider: Option<String>,
    /// Include `pdf_data` in the positions. Not supported by [`OrganizerOrders::list`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_data: Option<bool>,
}

/// Query parameters for [`Orders::get_with`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct GetOrder {
    /// Include canceled positions (of orders that are not fully canceled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_canceled_positions: Option<bool>,
    /// Include canceled fees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_canceled_fees: Option<bool>,
    /// Include `pdf_data` in the positions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_data: Option<bool>,
}

/// Query parameter of endpoints that may change quotas (`check_quotas`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct QuotaCheck {
    /// Whether to check quotas before committing changes (default `true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_quotas: Option<bool>,
}

impl QuotaCheck {
    /// Skips the quota check (`check_quotas=false`).
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            check_quotas: Some(false),
        }
    }
}

/// An invoice address in request bodies ([`CreateOrder`], [`UpdateOrder`]).
///
/// Set either `name` or `name_parts`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateInvoiceAddress {
    /// Customer company name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub company: Option<String>,
    /// Business or individual customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_business: Option<bool>,
    /// Customer name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<String>,
    /// Customer name decomposition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_parts: Option<BTreeMap<String, String>>,
    /// Customer street.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub street: Option<String>,
    /// Customer ZIP code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub zipcode: Option<String>,
    /// Customer city.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub city: Option<String>,
    /// Customer country code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub country: Option<String>,
    /// Customer state (ISO 3166-2 code).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub state: Option<String>,
    /// Customer's internal reference to be printed on the invoice.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub internal_reference: Option<String>,
    /// Custom invoice address field.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub custom_field: Option<String>,
    /// Customer VAT ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub vat_id: Option<String>,
    /// Set to `true` only if you validated the VAT ID yourself (enables reverse charge).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_id_validated: Option<bool>,
    /// Transmission channel for invoices (default `email`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transmission_type: Option<TransmissionType>,
    /// Transmission-channel specific information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transmission_info: Option<serde_json::Map<String, serde_json::Value>>,
}

/// An answer to a question in request bodies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateAnswer {
    /// Internal ID of the question.
    pub question: u64,
    /// The answer. When updating, `file:keep` keeps the current file of a file question.
    #[builder(into)]
    pub answer: String,
    /// Internal IDs of the selected options (only for choice questions).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub options: Vec<u64>,
}

/// Body of [`Orders::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateOrder {
    /// The customer email address.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub email: Option<Option<String>>,
    /// The customer phone number.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub phone: Option<Option<String>>,
    /// Whether the check-in app should show a warning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkin_attention: Option<bool>,
    /// Text shown when a ticket of this order is scanned.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub checkin_text: Option<Option<String>>,
    /// The locale used for communication with the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub locale: Option<String>,
    /// Internal comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub comment: Option<String>,
    /// API meta data (replaces the current value).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_meta: Option<serde_json::Map<String, serde_json::Value>>,
    /// Internal date for a custom follow-up action.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub custom_followup_at: Option<Option<Date>>,
    /// The full invoice address; `Some(None)` deletes the current address.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub invoice_address: Option<Option<CreateInvoiceAddress>>,
    /// Whether a pending order is treated like a paid order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_if_pending: Option<bool>,
    /// Payment deadline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<Timestamp>,
}

/// Body of [`Orders::mark_paid`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct MarkPaid {
    /// Whether to send the payment confirmation email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_email: Option<bool>,
}

/// Body of [`Orders::mark_canceled`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct MarkCanceled {
    /// Whether to notify the customer (default `true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_email: Option<bool>,
    /// A comment that may be used in the email template.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub comment: Option<String>,
    /// For paid orders: keep the order paid and replace all positions by this fee.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellation_fee: Option<Decimal>,
}

/// Body of [`Orders::extend`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ExtendOrder {
    /// New payment deadline. pretix uses the end of this day in the event's time zone.
    pub expires: Date,
    /// Extend even if this overbooks a quota (for expired orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

/// Body of [`Orders::deny`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct DenyOrder {
    /// Whether to notify the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_email: Option<bool>,
    /// The reason, shown to the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub comment: Option<String>,
}

/// Endpoints for the orders of an event. Created with [`EventScope::orders`].
#[derive(Debug, Clone)]
pub struct Orders {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Orders of this event.
    #[must_use]
    pub fn orders(&self) -> Orders {
        Orders {
            client: self.client(),
            path: self.path().join("orders"),
        }
    }
}

impl Orders {
    /// Lists orders.
    ///
    /// To fetch changes incrementally, pass the
    /// [`page_generated`](crate::ResponseMeta::page_generated) time of the first page as
    /// `modified_since` on the next sync.
    pub fn list(&self, params: &ListOrders) -> Request<Page<Order>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Like [`Orders::list`], only including or excluding the given fields (e.g.
    /// `positions.secret` or `fees`). The orders are therefore returned as JSON.
    pub fn list_with(
        &self,
        params: &ListOrders,
        fields: &FieldSelection,
    ) -> Request<Page<serde_json::Value>> {
        self.client
            .get(&self.path)
            .query(params)
            .query(fields)
            .page()
    }

    /// Fetches one order.
    pub fn get(&self, code: &str) -> Request<Order> {
        self.client.get(&self.path.join(code)).json()
    }

    /// Fetches one order with additional query parameters.
    pub fn get_with(&self, code: &str, params: &GetOrder) -> Request<Order> {
        self.client.get(&self.path.join(code)).query(params).json()
    }

    /// Creates an order (or simulates it with `simulate: true`).
    pub fn create(&self, body: &CreateOrder) -> Request<Order> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates an order.
    pub fn update(&self, code: &str, body: &UpdateOrder) -> Request<Order> {
        self.client
            .patch(&self.path.join(code))
            .json_body(body)
            .json()
    }

    /// Deletes an order. Only possible for test mode orders.
    pub fn delete(&self, code: &str) -> Request<()> {
        self.client.delete(&self.path.join(code)).empty()
    }

    /// Downloads the tickets of an order with the given output provider (e.g. `pdf`).
    ///
    /// Returns [`JobStatus::Pending`] while the file is being generated; use
    /// [`Request::wait`] to poll until it is ready.
    pub fn download(&self, code: &str, output: &str) -> Request<JobStatus<Download>> {
        self.client
            .get(&self.path.join(code).join("download").join(output))
            .download_job()
    }

    /// Generates new secrets for the order and all its positions.
    pub fn regenerate_secrets(&self, code: &str) -> Request<Order> {
        self.action(code, "regenerate_secrets").json()
    }

    /// Marks a pending or expired order as paid.
    pub fn mark_paid(&self, code: &str, body: &MarkPaid) -> Request<Order> {
        self.action(code, "mark_paid").json_body(body).json()
    }

    /// Cancels an order.
    pub fn mark_canceled(&self, code: &str, body: &MarkCanceled) -> Request<Order> {
        self.action(code, "mark_canceled").json_body(body).json()
    }

    /// Reactivates a canceled order.
    pub fn reactivate(&self, code: &str) -> Request<Order> {
        self.action(code, "reactivate").json()
    }

    /// Marks a paid order as unpaid.
    pub fn mark_pending(&self, code: &str) -> Request<Order> {
        self.action(code, "mark_pending").json()
    }

    /// Marks an unpaid order as expired.
    pub fn mark_expired(&self, code: &str) -> Request<Order> {
        self.action(code, "mark_expired").json()
    }

    /// Extends the payment deadline of a pending (or expired) order.
    pub fn extend(&self, code: &str, body: &ExtendOrder) -> Request<Order> {
        self.action(code, "extend").json_body(body).json()
    }

    /// Approves an order that is pending approval.
    pub fn approve(&self, code: &str) -> Request<Order> {
        self.action(code, "approve").json()
    }

    /// Denies an order that is pending approval.
    pub fn deny(&self, code: &str, body: &DenyOrder) -> Request<Order> {
        self.action(code, "deny").json_body(body).json()
    }

    /// Creates an invoice for an order that does not have one yet.
    pub fn create_invoice(&self, code: &str) -> Request<Invoice> {
        self.action(code, "create_invoice").json()
    }

    /// Sends an email with the link to the order page to the buyer.
    pub fn resend_link(&self, code: &str) -> Request<()> {
        self.action(code, "resend_link").empty()
    }

    /// Performs multiple changes to an order in one transaction.
    pub fn change(&self, code: &str, body: &ChangeOrder) -> Request<Order> {
        self.action(code, "change").json_body(body).json()
    }

    /// Like [`Orders::change`], with control over the quota check.
    pub fn change_with(
        &self,
        code: &str,
        body: &ChangeOrder,
        params: &QuotaCheck,
    ) -> Request<Order> {
        self.action(code, "change")
            .query(params)
            .json_body(body)
            .json()
    }

    /// Payments of the order `code`.
    #[must_use]
    pub fn payments(&self, code: &str) -> OrderPayments {
        OrderPayments::new(self.client.clone(), self.path.join(code).join("payments"))
    }

    /// Refunds of the order `code`.
    #[must_use]
    pub fn refunds(&self, code: &str) -> OrderRefunds {
        OrderRefunds::new(self.client.clone(), self.path.join(code).join("refunds"))
    }

    fn action(&self, code: &str, action: &str) -> crate::request::RequestBuilder {
        self.client.post(&self.path.join(code).join(action))
    }
}

/// Organizer-wide list of orders across all events. Created with
/// [`OrganizerScope::orders`].
#[derive(Debug, Clone)]
pub struct OrganizerOrders {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Orders of all events of this organizer.
    #[must_use]
    pub fn orders(&self) -> OrganizerOrders {
        OrganizerOrders {
            client: self.client(),
            path: self.path().join("orders"),
        }
    }
}

impl OrganizerOrders {
    /// Lists orders of all events (with sufficient permissions). `pdf_data` is not
    /// supported.
    pub fn list(&self, params: &ListOrders) -> Request<Page<Order>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Like [`OrganizerOrders::list`], only including or excluding the given fields.
    /// The orders are therefore returned as JSON.
    pub fn list_with(
        &self,
        params: &ListOrders,
        fields: &FieldSelection,
    ) -> Request<Page<serde_json::Value>> {
        self.client
            .get(&self.path)
            .query(params)
            .query(fields)
            .page()
    }
}
