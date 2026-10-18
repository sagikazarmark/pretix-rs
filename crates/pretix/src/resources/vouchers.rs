//! Vouchers: codes that unlock products, prices or reserved quota.
//!
//! <https://docs.pretix.eu/dev/api/resources/vouchers.html>

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Ordering;
use crate::types::{Decimal, string_enum};
use crate::{Client, EventScope, Page, Request};

/// A voucher.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct Voucher {
    /// Internal ID of the voucher.
    pub id: u64,
    /// When the voucher was created (guessed for vouchers created before pretix
    /// 2025.7).
    pub created: Timestamp,
    /// The code required to redeem the voucher.
    pub code: String,
    /// How many times the voucher can be redeemed.
    pub max_usages: u32,
    /// How many times the voucher has been redeemed.
    pub redeemed: u32,
    /// How many times the voucher must be redeemed on first usage.
    #[serde(default)]
    pub min_usages: Option<u32>,
    /// Expiration date.
    pub valid_until: Option<Timestamp>,
    /// Whether quota is blocked for this voucher.
    pub block_quota: bool,
    /// Whether the voucher can be redeemed even if the product is sold out.
    pub allow_ignore_quota: bool,
    /// How the voucher affects product prices.
    pub price_mode: PriceMode,
    /// The value (see `price_mode`).
    pub value: Option<Decimal>,
    /// ID of the item the voucher is restricted to.
    pub item: Option<u64>,
    /// ID of the variation the voucher is restricted to.
    pub variation: Option<u64>,
    /// ID of the quota the voucher is restricted to (alternative to `item` and
    /// `variation`).
    pub quota: Option<u64>,
    /// `seat_guid` of the seat the voucher is restricted to.
    pub seat: Option<String>,
    /// A string used for grouping vouchers.
    pub tag: String,
    /// An internal comment.
    pub comment: String,
    /// ID of the date inside an event series this voucher belongs to.
    pub subevent: Option<u64>,
    /// Whether the voucher allows buying products that are hidden without a voucher.
    pub show_hidden_items: bool,
    /// Whether all add-on products are included in the base price.
    pub all_addons_included: bool,
    /// Whether all bundled products are added without their designated price.
    pub all_bundles_included: bool,
    /// The budget the voucher may consume before being used up.
    pub budget: Option<Decimal>,
    /// The amount of budget already used up.
    pub budget_used: Decimal,
}

string_enum! {
    /// How a voucher affects product prices.
    pub enum PriceMode {
        /// No effect on price.
        None = "none",
        /// The price is set to `value`.
        Set = "set",
        /// The price is the original price minus `value`.
        Subtract = "subtract",
        /// The price is the original price reduced by `value` percent.
        Percent = "percent",
    }
}

string_enum! {
    /// Fields vouchers can be sorted by.
    pub enum VoucherOrdering {
        /// By ID (default).
        Id = "id",
        /// By code.
        Code = "code",
        /// By maximum usages.
        MaxUsages = "max_usages",
        /// By expiration date.
        ValidUntil = "valid_until",
        /// By value.
        Value = "value",
    }
}

/// Query parameters for [`Vouchers::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListVouchers {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<VoucherOrdering>>,
    /// Only the voucher with this code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub code: Option<String>,
    /// Only vouchers with this text in the code, tag or comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub search: Option<String>,
    /// Only vouchers with this maximum number of usages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_usages: Option<u32>,
    /// Only vouchers with this number of redemptions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redeemed: Option<u32>,
    /// Only vouchers that can (`true`) or cannot (`false`) still be redeemed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Only vouchers with this `block_quota` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_quota: Option<bool>,
    /// Only vouchers with this `allow_ignore_quota` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_ignore_quota: Option<bool>,
    /// Only vouchers with this price mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_mode: Option<PriceMode>,
    /// Only vouchers with this value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Decimal>,
    /// Only vouchers attached to this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<u64>,
    /// Only vouchers attached to this variation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// Only vouchers attached to this quota.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota: Option<u64>,
    /// Only vouchers with this tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub tag: Option<String>,
    /// Only vouchers of this sub-event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
}

/// Body of [`Vouchers::create`] and [`Vouchers::batch_create`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateVoucher {
    /// The code required to redeem the voucher. Generated by pretix if unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub code: Option<String>,
    /// How many times the voucher can be redeemed (default: 1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_usages: Option<u32>,
    /// How many times the voucher must be redeemed on first usage (default: 1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_usages: Option<u32>,
    /// Expiration date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<Timestamp>,
    /// Whether quota is blocked for this voucher.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_quota: Option<bool>,
    /// Whether the voucher can be redeemed even if the product is sold out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_ignore_quota: Option<bool>,
    /// How the voucher affects product prices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_mode: Option<PriceMode>,
    /// The value (see `price_mode`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Decimal>,
    /// ID of the item the voucher is restricted to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<u64>,
    /// ID of the variation the voucher is restricted to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// ID of the quota the voucher is restricted to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota: Option<u64>,
    /// `seat_guid` of the seat the voucher is restricted to.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub seat: Option<String>,
    /// A string used for grouping vouchers.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub tag: Option<String>,
    /// An internal comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub comment: Option<String>,
    /// ID of the date inside an event series this voucher belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// Whether the voucher allows buying products that are hidden without a voucher
    /// (default: `true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_hidden_items: Option<bool>,
    /// Whether all add-on products are included in the base price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_addons_included: Option<bool>,
    /// Whether all bundled products are added without their designated price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_bundles_included: Option<bool>,
    /// The budget the voucher may consume before being used up.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<Decimal>,
}

/// Body of [`Vouchers::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateVoucher {
    /// The code required to redeem the voucher.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub code: Option<String>,
    /// How many times the voucher can be redeemed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_usages: Option<u32>,
    /// How many times the voucher must be redeemed on first usage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_usages: Option<u32>,
    /// Expiration date.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub valid_until: Option<Option<Timestamp>>,
    /// Whether quota is blocked for this voucher.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_quota: Option<bool>,
    /// Whether the voucher can be redeemed even if the product is sold out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_ignore_quota: Option<bool>,
    /// How the voucher affects product prices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_mode: Option<PriceMode>,
    /// The value (see `price_mode`).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub value: Option<Option<Decimal>>,
    /// ID of the item the voucher is restricted to.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub item: Option<Option<u64>>,
    /// ID of the variation the voucher is restricted to.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub variation: Option<Option<u64>>,
    /// ID of the quota the voucher is restricted to.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub quota: Option<Option<u64>>,
    /// `seat_guid` of the seat the voucher is restricted to.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub seat: Option<Option<String>>,
    /// A string used for grouping vouchers.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub tag: Option<String>,
    /// An internal comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub comment: Option<String>,
    /// ID of the date inside an event series this voucher belongs to.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub subevent: Option<Option<u64>>,
    /// Whether the voucher allows buying products that are hidden without a voucher.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_hidden_items: Option<bool>,
    /// Whether all add-on products are included in the base price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_addons_included: Option<bool>,
    /// Whether all bundled products are added without their designated price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_bundles_included: Option<bool>,
    /// The budget the voucher may consume before being used up.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub budget: Option<Option<Decimal>>,
}

/// Endpoints for the vouchers of an event. Created with [`EventScope::vouchers`].
#[derive(Debug, Clone)]
pub struct Vouchers {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Vouchers of this event.
    #[must_use]
    pub fn vouchers(&self) -> Vouchers {
        Vouchers {
            client: self.client(),
            path: self.path().join("vouchers"),
        }
    }
}

impl Vouchers {
    /// Lists vouchers.
    pub fn list(&self, params: &ListVouchers) -> Request<Page<Voucher>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one voucher.
    pub fn get(&self, id: u64) -> Request<Voucher> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a voucher.
    pub fn create(&self, body: &CreateVoucher) -> Request<Voucher> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Creates multiple vouchers atomically.
    pub fn batch_create(&self, body: &[CreateVoucher]) -> Request<Vec<Voucher>> {
        self.client
            .post(&self.path.join("batch_create"))
            .json_body(body)
            .json()
    }

    /// Updates a voucher. `redeemed` cannot be changed.
    pub fn update(&self, id: u64, body: &UpdateVoucher) -> Request<Voucher> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a voucher. Redeemed vouchers cannot be deleted.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
