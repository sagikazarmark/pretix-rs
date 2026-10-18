//! Transactions: an immutable, filterable view into the history of orders, suitable for
//! financial reporting.
//!
//! <https://docs.pretix.eu/dev/api/resources/transactions.html>

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Ordering;
use crate::resources::orders::FeeType;
use crate::types::{Decimal, string_enum};
use crate::{Client, EventScope, OrganizerScope, Page, Request};

/// A transaction (the "debit" side of an order).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct Transaction {
    /// Internal ID of the transaction.
    pub id: u64,
    /// Code of the order the transaction was created from.
    pub order: String,
    /// Event slug. Only included in organizer-level lists.
    #[serde(default)]
    pub event: Option<String>,
    /// Creation time of the transaction in the database.
    pub created: Timestamp,
    /// Time at which the transaction is financially relevant.
    pub datetime: Timestamp,
    /// Number of the position within the order (`None` for fees).
    #[serde(default)]
    pub positionid: Option<u32>,
    /// Number of items purchased, negative for cancellations.
    pub count: i64,
    /// ID of the purchased item (`None` for fees).
    #[serde(default)]
    pub item: Option<u64>,
    /// ID of the purchased variation.
    #[serde(default)]
    pub variation: Option<u64>,
    /// ID of the event series date.
    #[serde(default)]
    pub subevent: Option<u64>,
    /// Gross price.
    pub price: Decimal,
    /// Tax rate.
    pub tax_rate: Decimal,
    /// ID of the tax rule.
    #[serde(default)]
    pub tax_rule: Option<u64>,
    /// Tax code.
    #[serde(default)]
    pub tax_code: Option<String>,
    /// Computed tax value.
    pub tax_value: Decimal,
    /// Type of fee (`None` for products).
    #[serde(default)]
    pub fee_type: Option<FeeType>,
    /// Additional classification of the fee (`None` for products).
    #[serde(default)]
    pub internal_type: Option<String>,
}

string_enum! {
    /// Fields transactions can be sorted by.
    pub enum TransactionOrdering {
        /// By financially relevant time.
        Datetime = "datetime",
        /// By creation time.
        Created = "created",
        /// By ID.
        Id = "id",
    }
}

/// Query parameters for [`Transactions::list`] and [`OrganizerTransactions::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListTransactions {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<TransactionOrdering>>,
    /// Only transactions of this event slug. Only supported by
    /// [`OrganizerTransactions::list`].
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub event: Option<String>,
    /// Only transactions of this order code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub order: Option<String>,
    /// Only transactions with a `datetime` at or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime_since: Option<Timestamp>,
    /// Only transactions with a `datetime` before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime_before: Option<Timestamp>,
    /// Only transactions created at or after this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_since: Option<Timestamp>,
    /// Only transactions created before this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_before: Option<Timestamp>,
    /// Only transactions of this item ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<u64>,
    /// Only transactions of one of these item IDs.
    #[serde(
        rename = "item__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub item_in: Vec<u64>,
    /// Only transactions of this variation ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// Only transactions of one of these variation IDs.
    #[serde(
        rename = "variation__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub variation_in: Vec<u64>,
    /// Only transactions of this sub-event ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// Only transactions of one of these sub-event IDs.
    #[serde(
        rename = "subevent__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub subevent_in: Vec<u64>,
    /// Only transactions with this tax rule ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_rule: Option<u64>,
    /// Only transactions with one of these tax rule IDs.
    #[serde(
        rename = "tax_rule__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub tax_rule_in: Vec<u64>,
    /// Only transactions with this tax code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub tax_code: Option<String>,
    /// Only transactions with one of these tax codes.
    #[serde(
        rename = "tax_code__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub tax_code_in: Vec<String>,
    /// Only transactions with this tax rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_rate: Option<Decimal>,
    /// Only transactions with one of these tax rates.
    #[serde(
        rename = "tax_rate__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub tax_rate_in: Vec<Decimal>,
    /// Only transactions with this fee type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_type: Option<FeeType>,
    /// Only transactions with one of these fee types.
    #[serde(
        rename = "fee_type__in",
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "crate::serde_util::comma"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[builder(default)]
    pub fee_type_in: Vec<FeeType>,
}

/// Endpoints for the transactions of an event. Created with
/// [`EventScope::transactions`].
#[derive(Debug, Clone)]
pub struct Transactions {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Transactions of this event.
    #[must_use]
    pub fn transactions(&self) -> Transactions {
        Transactions {
            client: self.client(),
            path: self.path().join("transactions"),
        }
    }
}

impl Transactions {
    /// Lists transactions.
    pub fn list(&self, params: &ListTransactions) -> Request<Page<Transaction>> {
        self.client.get(&self.path).query(params).page()
    }
}

/// Organizer-wide list of transactions across all events. Created with
/// [`OrganizerScope::transactions`].
#[derive(Debug, Clone)]
pub struct OrganizerTransactions {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Transactions of all events of this organizer.
    #[must_use]
    pub fn transactions(&self) -> OrganizerTransactions {
        OrganizerTransactions {
            client: self.client(),
            path: self.path().join("transactions"),
        }
    }
}

impl OrganizerTransactions {
    /// Lists transactions of all events (with sufficient permissions).
    pub fn list(&self, params: &ListTransactions) -> Request<Page<Transaction>> {
        self.client.get(&self.path).query(params).page()
    }
}
