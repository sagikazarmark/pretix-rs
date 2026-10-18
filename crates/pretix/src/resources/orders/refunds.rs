//! Order refund endpoints.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::{OrderRefund, RefundSource, RefundState};
use crate::path::Path;
use crate::types::Decimal;
use crate::{Client, ListParams, Page, Request};

/// Body of [`OrderRefunds::create`].
///
/// Recommended only for refunds with payment provider `manual`; amounts are not
/// checked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateOrderRefund {
    /// Refund state.
    pub state: RefundState,
    /// How this refund has been created.
    pub source: RefundSource,
    /// Refund amount.
    pub amount: Decimal,
    /// `local_id` of the payment this refund belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment: Option<u64>,
    /// Completion time of the refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_date: Option<Timestamp>,
    /// Reason for the refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub comment: Option<String>,
    /// Identification string of the payment provider.
    #[builder(into)]
    pub provider: String,
    /// Refund-specific information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<serde_json::Value>,
    /// Whether to cancel the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_canceled: Option<bool>,
    /// Whether to mark the order as pending.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_pending: Option<bool>,
}

/// Body of [`OrderRefunds::process`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ProcessRefund {
    /// Cancel the order (`true`) or mark it as pending (`false`).
    pub mark_canceled: bool,
}

/// Endpoints for the refunds of one order. Created with
/// [`Orders::refunds`](super::Orders::refunds).
#[derive(Debug, Clone)]
pub struct OrderRefunds {
    client: Client,
    path: Path,
}

impl OrderRefunds {
    pub(super) fn new(client: Client, path: Path) -> Self {
        Self { client, path }
    }

    /// Lists the refunds of the order.
    pub fn list(&self, params: &ListParams) -> Request<Page<OrderRefund>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one refund by its order-local ID.
    pub fn get(&self, local_id: u64) -> Request<OrderRefund> {
        self.client.get(&self.path.join(local_id)).json()
    }

    /// Creates a refund manually.
    pub fn create(&self, body: &CreateOrderRefund) -> Request<OrderRefund> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Marks a `transit` or `created` refund as done.
    pub fn done(&self, local_id: u64) -> Request<OrderRefund> {
        self.client
            .post(&self.path.join(local_id).join("done"))
            .json()
    }

    /// Acts on an `external` refund by canceling the order or marking it as pending.
    pub fn process(&self, local_id: u64, body: &ProcessRefund) -> Request<OrderRefund> {
        self.client
            .post(&self.path.join(local_id).join("process"))
            .json_body(body)
            .json()
    }

    /// Marks a `transit`, `external` or `created` refund as canceled.
    pub fn cancel(&self, local_id: u64) -> Request<OrderRefund> {
        self.client
            .post(&self.path.join(local_id).join("cancel"))
            .json()
    }
}
