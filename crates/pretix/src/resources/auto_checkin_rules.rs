//! Auto check-in rules: automatically check in tickets after purchase (requires the
//! `pretix.plugins.autocheckin` plugin).
//!
//! <https://docs.pretix.eu/dev/api/resources/auto_checkin_rules.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::types::string_enum;
use crate::{Client, EventScope, ListParams, Page, Request};

/// An auto check-in rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct AutoCheckinRule {
    /// Internal ID of the rule.
    pub id: u64,
    /// ID of the check-in list to check the ticket in on (`None` = all matching lists).
    pub list: Option<u64>,
    /// When the rule is evaluated.
    pub mode: AutoCheckinMode,
    /// Whether the rule applies to tickets sold on all sales channels.
    pub all_sales_channels: bool,
    /// Sales channel identifiers the rule applies to, if not all.
    #[serde(default)]
    pub limit_sales_channels: Vec<String>,
    /// Whether the rule applies to all products and variations.
    pub all_products: bool,
    /// Item IDs the rule applies to (including all their variations), if not all.
    #[serde(default)]
    pub limit_products: Vec<u64>,
    /// Variation IDs the rule applies to, if not all products.
    #[serde(default)]
    pub limit_variations: Vec<u64>,
    /// Whether the rule applies to tickets paid with all payment methods.
    pub all_payment_methods: bool,
    /// Payment method identifiers the rule applies to, if not all.
    #[serde(default)]
    pub limit_payment_methods: Vec<String>,
}

string_enum! {
    /// When an auto check-in rule is evaluated.
    pub enum AutoCheckinMode {
        /// Right after an order has been placed.
        Placed = "placed",
        /// After the order has been fully paid.
        Paid = "paid",
    }
}

/// Body of [`AutoCheckinRules::create`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateAutoCheckinRule {
    /// ID of the check-in list (`None` = all matching lists).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<u64>,
    /// When the rule is evaluated.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub mode: Option<AutoCheckinMode>,
    /// Whether the rule applies to all sales channels (default `true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_sales_channels: Option<bool>,
    /// Sales channel identifiers, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_sales_channels: Option<Vec<String>>,
    /// Whether the rule applies to all products (default `true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_products: Option<bool>,
    /// Item IDs, if not all products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_products: Option<Vec<u64>>,
    /// Variation IDs, if not all products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_variations: Option<Vec<u64>>,
    /// Whether the rule applies to all payment methods (default `true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_payment_methods: Option<bool>,
    /// Payment method identifiers, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_payment_methods: Option<Vec<String>>,
}

/// Body of [`AutoCheckinRules::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateAutoCheckinRule {
    /// ID of the check-in list (`Some(None)` = all matching lists).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub list: Option<Option<u64>>,
    /// When the rule is evaluated.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub mode: Option<AutoCheckinMode>,
    /// Whether the rule applies to all sales channels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_sales_channels: Option<bool>,
    /// Sales channel identifiers, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_sales_channels: Option<Vec<String>>,
    /// Whether the rule applies to all products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_products: Option<bool>,
    /// Item IDs, if not all products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_products: Option<Vec<u64>>,
    /// Variation IDs, if not all products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_variations: Option<Vec<u64>>,
    /// Whether the rule applies to all payment methods.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_payment_methods: Option<bool>,
    /// Payment method identifiers, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_payment_methods: Option<Vec<String>>,
}

/// Endpoints for the auto check-in rules of an event. Created with
/// [`EventScope::auto_checkin_rules`].
#[derive(Debug, Clone)]
pub struct AutoCheckinRules {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Auto check-in rules of this event.
    #[must_use]
    pub fn auto_checkin_rules(&self) -> AutoCheckinRules {
        AutoCheckinRules {
            client: self.client(),
            path: self.path().join("auto_checkin_rules"),
        }
    }
}

impl AutoCheckinRules {
    /// Lists rules.
    pub fn list(&self, params: &ListParams) -> Request<Page<AutoCheckinRule>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one rule.
    pub fn get(&self, id: u64) -> Request<AutoCheckinRule> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a rule.
    pub fn create(&self, body: &CreateAutoCheckinRule) -> Request<AutoCheckinRule> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a rule.
    pub fn update(&self, id: u64, body: &UpdateAutoCheckinRule) -> Request<AutoCheckinRule> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a rule.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
