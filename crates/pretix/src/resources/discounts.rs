//! Discounts: automatic price reductions for carts matching a set of conditions.
//!
//! <https://docs.pretix.eu/dev/api/resources/discounts.html>

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Ordering;
use crate::types::{Decimal, string_enum};
use crate::{Client, EventScope, Page, Request};

/// A discount rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Discount {
    /// Internal ID of the discount rule.
    pub id: u64,
    /// The discount is ignored if this is `false`.
    pub active: bool,
    /// A name for the rule used in the backend.
    pub internal_name: String,
    /// Used for sorting the rules, which are applied in order.
    pub position: i64,
    /// Whether the discount is available on all sales channels that support discounts.
    pub all_sales_channels: bool,
    /// Sales channel identifiers the discount is available on, if not all.
    #[serde(default)]
    pub limit_sales_channels: Vec<String>,
    /// **Deprecated**: use `all_sales_channels` and `limit_sales_channels`.
    #[serde(default)]
    pub sales_channels: Vec<String>,
    /// The first time at which this discount can be applied.
    pub available_from: Option<Timestamp>,
    /// The last time at which this discount can be applied.
    pub available_until: Option<Timestamp>,
    /// How the discount is handled in an event series.
    pub subevent_mode: SubeventMode,
    /// The first start of a sub-event this discount can be applied to.
    pub subevent_date_from: Option<Timestamp>,
    /// The last start of a sub-event this discount can be applied to.
    pub subevent_date_until: Option<Timestamp>,
    /// Whether the condition applies to all items.
    pub condition_all_products: bool,
    /// Item IDs the condition applies to, if not all.
    #[serde(default)]
    pub condition_limit_products: Vec<u64>,
    /// Whether the condition applies to add-on products as well.
    pub condition_apply_to_addons: bool,
    /// Whether products discounted by a voucher are ignored by the condition.
    pub condition_ignore_voucher_discounted: bool,
    /// Minimum number of matching products to activate the discount.
    pub condition_min_count: u32,
    /// Minimum value of matching products to activate the discount.
    pub condition_min_value: Decimal,
    /// Percentage of price reduction for matching products.
    pub benefit_discount_matching_percent: Decimal,
    /// If greater than 0, the discount only applies to the cheapest this many matches.
    pub benefit_only_apply_to_cheapest_n_matches: Option<u32>,
    /// Whether the benefit applies to the same items as the condition.
    pub benefit_same_products: bool,
    /// Item IDs the benefit applies to, if not `benefit_same_products`.
    #[serde(default)]
    pub benefit_limit_products: Vec<u64>,
    /// Whether the benefit applies to add-on products as well (only used if not
    /// `benefit_same_products`).
    pub benefit_apply_to_addons: bool,
    /// Whether products discounted by a voucher are excluded from the benefit (only
    /// used if not `benefit_same_products`).
    pub benefit_ignore_voucher_discounted: bool,
}

string_enum! {
    /// How a discount is handled in an event series.
    pub enum SubeventMode {
        /// No special effect.
        Mixed = "mixed",
        /// Only applied to groups within the same date.
        Same = "same",
        /// Only applied to groups with no two same dates.
        Distinct = "distinct",
    }
}

string_enum! {
    /// Fields discounts can be sorted by.
    pub enum DiscountOrdering {
        /// By ID.
        Id = "id",
        /// By position (default).
        Position = "position",
    }
}

/// Query parameters for [`Discounts::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct ListDiscounts {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<DiscountOrdering>>,
    /// Only discounts with this `active` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

/// Body of [`Discounts::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateDiscount {
    /// A name for the rule used in the backend.
    #[builder(into)]
    pub internal_name: String,
    /// The discount is ignored if this is `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Used for sorting the rules, which are applied in order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// Whether the discount is available on all sales channels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_sales_channels: Option<bool>,
    /// Sales channel identifiers the discount is available on, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_sales_channels: Option<Vec<String>>,
    /// The first time at which this discount can be applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_from: Option<Timestamp>,
    /// The last time at which this discount can be applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_until: Option<Timestamp>,
    /// How the discount is handled in an event series.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent_mode: Option<SubeventMode>,
    /// The first start of a sub-event this discount can be applied to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent_date_from: Option<Timestamp>,
    /// The last start of a sub-event this discount can be applied to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent_date_until: Option<Timestamp>,
    /// Whether the condition applies to all items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_all_products: Option<bool>,
    /// Item IDs the condition applies to, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_limit_products: Option<Vec<u64>>,
    /// Whether the condition applies to add-on products as well.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_apply_to_addons: Option<bool>,
    /// Whether products discounted by a voucher are ignored by the condition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_ignore_voucher_discounted: Option<bool>,
    /// Minimum number of matching products to activate the discount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_min_count: Option<u32>,
    /// Minimum value of matching products to activate the discount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_min_value: Option<Decimal>,
    /// Percentage of price reduction for matching products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit_discount_matching_percent: Option<Decimal>,
    /// If greater than 0, the discount only applies to the cheapest this many matches.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit_only_apply_to_cheapest_n_matches: Option<u32>,
    /// Whether the benefit applies to the same items as the condition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit_same_products: Option<bool>,
    /// Item IDs the benefit applies to, if not `benefit_same_products`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit_limit_products: Option<Vec<u64>>,
    /// Whether the benefit applies to add-on products as well.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit_apply_to_addons: Option<bool>,
    /// Whether products discounted by a voucher are excluded from the benefit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit_ignore_voucher_discounted: Option<bool>,
}

/// Body of [`Discounts::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateDiscount {
    /// A name for the rule used in the backend.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub internal_name: Option<String>,
    /// The discount is ignored if this is `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Used for sorting the rules, which are applied in order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// Whether the discount is available on all sales channels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_sales_channels: Option<bool>,
    /// Sales channel identifiers the discount is available on, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_sales_channels: Option<Vec<String>>,
    /// The first time at which this discount can be applied.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub available_from: Option<Option<Timestamp>>,
    /// The last time at which this discount can be applied.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub available_until: Option<Option<Timestamp>>,
    /// How the discount is handled in an event series.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent_mode: Option<SubeventMode>,
    /// The first start of a sub-event this discount can be applied to.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub subevent_date_from: Option<Option<Timestamp>>,
    /// The last start of a sub-event this discount can be applied to.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub subevent_date_until: Option<Option<Timestamp>>,
    /// Whether the condition applies to all items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_all_products: Option<bool>,
    /// Item IDs the condition applies to, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_limit_products: Option<Vec<u64>>,
    /// Whether the condition applies to add-on products as well.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_apply_to_addons: Option<bool>,
    /// Whether products discounted by a voucher are ignored by the condition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_ignore_voucher_discounted: Option<bool>,
    /// Minimum number of matching products to activate the discount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_min_count: Option<u32>,
    /// Minimum value of matching products to activate the discount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_min_value: Option<Decimal>,
    /// Percentage of price reduction for matching products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit_discount_matching_percent: Option<Decimal>,
    /// If greater than 0, the discount only applies to the cheapest this many matches.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub benefit_only_apply_to_cheapest_n_matches: Option<Option<u32>>,
    /// Whether the benefit applies to the same items as the condition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit_same_products: Option<bool>,
    /// Item IDs the benefit applies to, if not `benefit_same_products`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit_limit_products: Option<Vec<u64>>,
    /// Whether the benefit applies to add-on products as well.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit_apply_to_addons: Option<bool>,
    /// Whether products discounted by a voucher are excluded from the benefit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit_ignore_voucher_discounted: Option<bool>,
}

/// Endpoints for the discounts of an event. Created with [`EventScope::discounts`].
#[derive(Debug, Clone)]
pub struct Discounts {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Discounts of this event.
    #[must_use]
    pub fn discounts(&self) -> Discounts {
        Discounts {
            client: self.client(),
            path: self.path().join("discounts"),
        }
    }
}

impl Discounts {
    /// Lists discounts.
    pub fn list(&self, params: &ListDiscounts) -> Request<Page<Discount>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one discount.
    pub fn get(&self, id: u64) -> Request<Discount> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a discount.
    pub fn create(&self, body: &CreateDiscount) -> Request<Discount> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a discount.
    pub fn update(&self, id: u64, body: &UpdateDiscount) -> Request<Discount> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a discount.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
