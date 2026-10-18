use std::collections::BTreeMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::{AvailabilityMode, Items};
use crate::path::Path;
use crate::types::{Decimal, LocalizedString};
use crate::{Client, Page, Request};

/// A variation of an item, e.g. a size or color.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ItemVariation {
    /// Internal ID of the variation.
    pub id: u64,
    /// The "name" of the variation.
    pub value: LocalizedString,
    /// The price set directly for this variation.
    pub default_price: Option<Decimal>,
    /// The price used for this variation: `default_price` if set, otherwise the item's
    /// `default_price`.
    pub price: Decimal,
    /// A suggested price, used as a default value if the item has `free_price` set.
    #[serde(default)]
    pub free_price_suggestion: Option<Decimal>,
    /// An original price, shown for comparison, not used for price calculations.
    pub original_price: Option<Decimal>,
    /// Whether this variation is sold and shown.
    pub active: bool,
    /// A public description (might include Markdown).
    pub description: Option<LocalizedString>,
    /// Used for sorting.
    pub position: i64,
    /// Whether the check-in app should show a warning when this variation is scanned.
    pub checkin_attention: bool,
    /// Text shown when a ticket of this variation is scanned.
    #[serde(default)]
    pub checkin_text: Option<String>,
    /// Whether orders with this variation need approval by the organizer.
    pub require_approval: bool,
    /// Whether booking this variation requires an active membership.
    pub require_membership: bool,
    /// Whether this variation is hidden from users without a valid membership (if
    /// `require_membership` is set).
    #[serde(default)]
    pub require_membership_hidden: bool,
    /// IDs of the membership types valid if `require_membership` is set.
    #[serde(default)]
    pub require_membership_types: Vec<u64>,
    /// Whether the variation is available on all sales channels.
    pub all_sales_channels: bool,
    /// Sales channel identifiers the variation is available on, if not all. A sales
    /// channel needs to be on the item's list too.
    #[serde(default)]
    pub limit_sales_channels: Vec<String>,
    /// **Deprecated**: use `all_sales_channels` and `limit_sales_channels`.
    #[serde(default)]
    pub sales_channels: Vec<String>,
    /// The first time at which this variation can be bought.
    pub available_from: Option<Timestamp>,
    /// How the variation is shown before `available_from`.
    pub available_from_mode: AvailabilityMode,
    /// The last time at which this variation can be bought.
    pub available_until: Option<Timestamp>,
    /// How the variation is shown after `available_until`.
    pub available_until_mode: AvailabilityMode,
    /// Whether this variation is only shown during voucher redemption.
    pub hide_without_voucher: bool,
    /// Values of item meta data properties.
    #[serde(default)]
    pub meta_data: BTreeMap<String, String>,
}

/// Query parameters for [`ItemVariations::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListItemVariations {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Substring search in the variation's value.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub search: Option<String>,
    /// Only variations with this `active` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

/// Body of [`ItemVariations::create`], also used for the initial variations in
/// [`CreateItem`](super::CreateItem).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateItemVariation {
    /// The "name" of the variation.
    #[builder(into)]
    pub value: LocalizedString,
    /// The price set directly for this variation (unset to use the item's price).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_price: Option<Decimal>,
    /// A suggested price, used as a default value if the item has `free_price` set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_price_suggestion: Option<Decimal>,
    /// An original price, shown for comparison.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_price: Option<Decimal>,
    /// Whether this variation is sold and shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// A public description (might include Markdown).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub description: Option<LocalizedString>,
    /// Used for sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// Whether the check-in app should show a warning when this variation is scanned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkin_attention: Option<bool>,
    /// Text shown when a ticket of this variation is scanned.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub checkin_text: Option<String>,
    /// Whether orders with this variation need approval by the organizer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<bool>,
    /// Whether booking this variation requires an active membership.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_membership: Option<bool>,
    /// Whether this variation is hidden from users without a valid membership.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_membership_hidden: Option<bool>,
    /// IDs of the membership types valid if `require_membership` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_membership_types: Option<Vec<u64>>,
    /// Whether the variation is available on all sales channels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_sales_channels: Option<bool>,
    /// Sales channel identifiers the variation is available on, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_sales_channels: Option<Vec<String>>,
    /// The first time at which this variation can be bought.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_from: Option<Timestamp>,
    /// How the variation is shown before `available_from`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub available_from_mode: Option<AvailabilityMode>,
    /// The last time at which this variation can be bought.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_until: Option<Timestamp>,
    /// How the variation is shown after `available_until`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub available_until_mode: Option<AvailabilityMode>,
    /// Whether this variation is only shown during voucher redemption.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_without_voucher: Option<bool>,
    /// Values of item meta data properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data: Option<BTreeMap<String, String>>,
}

/// Body of [`ItemVariations::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateItemVariation {
    /// The "name" of the variation.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub value: Option<LocalizedString>,
    /// The price set directly for this variation (`Some(None)` uses the item's price).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub default_price: Option<Option<Decimal>>,
    /// A suggested price, used as a default value if the item has `free_price` set.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub free_price_suggestion: Option<Option<Decimal>>,
    /// An original price, shown for comparison.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub original_price: Option<Option<Decimal>>,
    /// Whether this variation is sold and shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// A public description (might include Markdown).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub description: Option<Option<LocalizedString>>,
    /// Used for sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// Whether the check-in app should show a warning when this variation is scanned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkin_attention: Option<bool>,
    /// Text shown when a ticket of this variation is scanned.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub checkin_text: Option<Option<String>>,
    /// Whether orders with this variation need approval by the organizer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<bool>,
    /// Whether booking this variation requires an active membership.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_membership: Option<bool>,
    /// Whether this variation is hidden from users without a valid membership.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_membership_hidden: Option<bool>,
    /// IDs of the membership types valid if `require_membership` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_membership_types: Option<Vec<u64>>,
    /// Whether the variation is available on all sales channels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_sales_channels: Option<bool>,
    /// Sales channel identifiers the variation is available on, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_sales_channels: Option<Vec<String>>,
    /// The first time at which this variation can be bought.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub available_from: Option<Option<Timestamp>>,
    /// How the variation is shown before `available_from`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub available_from_mode: Option<AvailabilityMode>,
    /// The last time at which this variation can be bought.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub available_until: Option<Option<Timestamp>>,
    /// How the variation is shown after `available_until`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub available_until_mode: Option<AvailabilityMode>,
    /// Whether this variation is only shown during voucher redemption.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_without_voucher: Option<bool>,
    /// Values of item meta data properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data: Option<BTreeMap<String, String>>,
}

/// Endpoints for the variations of an item. Created with [`Items::variations`].
#[derive(Debug, Clone)]
pub struct ItemVariations {
    client: Client,
    path: Path,
}

impl Items {
    /// Variations of the item with ID `item`.
    #[must_use]
    pub fn variations(&self, item: u64) -> ItemVariations {
        ItemVariations {
            client: self.client.clone(),
            path: self.path.join(item).join("variations"),
        }
    }
}

impl ItemVariations {
    /// Lists variations.
    pub fn list(&self, params: &ListItemVariations) -> Request<Page<ItemVariation>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one variation.
    pub fn get(&self, id: u64) -> Request<ItemVariation> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a variation.
    pub fn create(&self, body: &CreateItemVariation) -> Request<ItemVariation> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a variation. `price` cannot be changed.
    pub fn update(&self, id: u64, body: &UpdateItemVariation) -> Request<ItemVariation> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a variation.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
