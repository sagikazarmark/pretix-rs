//! Order payment endpoints.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::{OrderPayment, OrderRefund, PaymentState};
use crate::path::Path;
use crate::types::Decimal;
use crate::{Client, ListParams, Page, Request};

/// Body of [`OrderPayments::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateOrderPayment {
    /// Payment state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<PaymentState>,
    /// Payment amount.
    pub amount: Decimal,
    /// Completion time of the payment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_date: Option<Timestamp>,
    /// Identification string of the payment provider.
    #[builder(into)]
    pub provider: String,
    /// Internal `info` value of the payment. Informational only: the payment provider
    /// is not called.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<serde_json::Value>,
    /// Whether to send the payment confirmation email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_email: Option<bool>,
}

/// Body of [`OrderPayments::confirm`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ConfirmPayment {
    /// Whether to send the payment confirmation email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_email: Option<bool>,
    /// Confirm even if quotas are exceeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

/// Body of [`OrderPayments::refund`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct RefundPayment {
    /// Amount to refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Decimal>,
    /// Reason for the refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub comment: Option<String>,
    /// Whether to cancel the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_canceled: Option<bool>,
}

/// Endpoints for the payments of one order. Created with
/// [`Orders::payments`](super::Orders::payments).
#[derive(Debug, Clone)]
pub struct OrderPayments {
    client: Client,
    path: Path,
}

impl OrderPayments {
    pub(super) fn new(client: Client, path: Path) -> Self {
        Self { client, path }
    }

    /// Lists the payments of the order.
    pub fn list(&self, params: &ListParams) -> Request<Page<OrderPayment>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one payment by its order-local ID.
    pub fn get(&self, local_id: u64) -> Request<OrderPayment> {
        self.client.get(&self.path.join(local_id)).json()
    }

    /// Creates a payment.
    pub fn create(&self, body: &CreateOrderPayment) -> Request<OrderPayment> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Marks a `pending` or `created` payment as confirmed.
    pub fn confirm(&self, local_id: u64, body: &ConfirmPayment) -> Request<OrderPayment> {
        self.client
            .post(&self.path.join(local_id).join("confirm"))
            .json_body(body)
            .json()
    }

    /// Marks a `pending` or `created` payment as canceled.
    pub fn cancel(&self, local_id: u64) -> Request<OrderPayment> {
        self.client
            .post(&self.path.join(local_id).join("cancel"))
            .json()
    }

    /// Creates and executes a manual refund of a `confirmed` payment. Returns the
    /// refund.
    pub fn refund(&self, local_id: u64, body: &RefundPayment) -> Request<OrderRefund> {
        self.client
            .post(&self.path.join(local_id).join("refund"))
            .json_body(body)
            .json()
    }
}
