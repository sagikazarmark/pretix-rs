//! Items (products), with their variations, bundles, add-ons and program times.
//!
//! - <https://docs.pretix.eu/dev/api/resources/items.html>
//! - <https://docs.pretix.eu/dev/api/resources/item_variations.html>
//! - <https://docs.pretix.eu/dev/api/resources/item_bundles.html>
//! - <https://docs.pretix.eu/dev/api/resources/item_add-ons.html>
//! - <https://docs.pretix.eu/dev/api/resources/item_program_times.html>

use std::collections::BTreeMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Ordering;
use crate::resources::reusable_media::MediumType;
use crate::types::{Decimal, FileId, LocalizedString, string_enum};
use crate::{Client, EventScope, Page, Request};

mod addons;
mod bundles;
mod program_times;
mod variations;

pub use addons::{CreateItemAddOn, ItemAddOn, ItemAddOns, UpdateItemAddOn};
pub use bundles::{CreateItemBundle, ItemBundle, ItemBundles, UpdateItemBundle};
pub use program_times::{
    CreateItemProgramTime, ItemProgramTime, ItemProgramTimes, UpdateItemProgramTime,
};
pub use variations::{
    CreateItemVariation, ItemVariation, ItemVariations, ListItemVariations, UpdateItemVariation,
};

/// An item (product).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Item {
    /// Internal ID of the item.
    pub id: u64,
    /// The item's visible name.
    pub name: LocalizedString,
    /// An optional name that is only used in the backend.
    pub internal_name: Option<String>,
    /// The price applied if not overridden by variations or other options.
    pub default_price: Decimal,
    /// ID of the category this item belongs to.
    pub category: Option<u64>,
    /// Whether the item is shown in public lists and sold.
    pub active: bool,
    /// A public description (might include Markdown).
    pub description: Option<LocalizedString>,
    /// Whether customers can choose the price (at least `default_price`).
    pub free_price: bool,
    /// A suggested price, used as a default value if `free_price` is set.
    #[serde(default)]
    pub free_price_suggestion: Option<Decimal>,
    /// The VAT rate applied to this item (read-only, set through `tax_rule`).
    pub tax_rate: Decimal,
    /// ID of the applied tax rule.
    pub tax_rule: Option<u64>,
    /// Whether the item grants admission to the event (e.g. primary tickets).
    pub admission: bool,
    /// Whether the item requires personalization according to event settings.
    pub personalized: bool,
    /// Used for sorting.
    pub position: i64,
    /// URL of the product picture.
    pub picture: Option<String>,
    /// Whether the item is available on all sales channels.
    pub all_sales_channels: bool,
    /// Sales channel identifiers the item is available on, if not all.
    #[serde(default)]
    pub limit_sales_channels: Vec<String>,
    /// **Deprecated**: use `all_sales_channels` and `limit_sales_channels`.
    #[serde(default)]
    pub sales_channels: Vec<String>,
    /// The first time at which this item can be bought.
    pub available_from: Option<Timestamp>,
    /// How the item is shown before `available_from`.
    pub available_from_mode: AvailabilityMode,
    /// The last time at which this item can be bought.
    pub available_until: Option<Timestamp>,
    /// How the item is shown after `available_until`.
    pub available_until_mode: AvailabilityMode,
    /// **Deprecated**: ID of a quota; the item is hidden as long as that quota is
    /// available.
    pub hidden_if_available: Option<u64>,
    /// ID of another item; this item is hidden as long as that item is available.
    #[serde(default)]
    pub hidden_if_item_available: Option<u64>,
    /// How the item is shown while `hidden_if_item_available` is available. Defaults to
    /// [`AvailabilityMode::Hide`] if missing.
    #[serde(default = "default_hidden_if_item_available_mode")]
    pub hidden_if_item_available_mode: AvailabilityMode,
    /// Whether the item can only be bought with a voucher assigned to it.
    pub require_voucher: bool,
    /// Whether the item is only shown during voucher redemption.
    pub hide_without_voucher: bool,
    /// Whether customers can cancel orders containing this item.
    pub allow_cancel: bool,
    /// Minimum number of this item per order.
    pub min_per_order: Option<u32>,
    /// Maximum number of this item per order.
    pub max_per_order: Option<u32>,
    /// Whether the check-in app should show a warning when this item is scanned.
    pub checkin_attention: bool,
    /// Text shown when a ticket of this item is scanned.
    #[serde(default)]
    pub checkin_text: Option<String>,
    /// An original price, shown for comparison, not used for price calculations.
    pub original_price: Option<Decimal>,
    /// Whether orders with this item need approval by the organizer.
    pub require_approval: bool,
    /// Whether this item is only available as part of bundles.
    pub require_bundling: bool,
    /// Whether booking this item requires an active membership.
    pub require_membership: bool,
    /// Whether this item is hidden from users without a valid membership (if
    /// `require_membership` is set).
    #[serde(default)]
    pub require_membership_hidden: bool,
    /// IDs of the membership types valid if `require_membership` is set.
    #[serde(default)]
    pub require_membership_types: Vec<u64>,
    /// ID of the membership type created when purchasing this item.
    pub grant_membership_type: Option<u64>,
    /// Whether the granted membership's term is the purchased (sub-)event's duration.
    pub grant_membership_duration_like_event: bool,
    /// Days of the granted membership's term (if not like the event).
    pub grant_membership_duration_days: u32,
    /// Calendar months of the granted membership's term (if not like the event).
    pub grant_membership_duration_months: u32,
    /// Special validity of issued tickets (`None` = follow event configuration).
    #[serde(default)]
    pub validity_mode: Option<ValidityMode>,
    /// Start of validity if `validity_mode` is `fixed`.
    pub validity_fixed_from: Option<Timestamp>,
    /// End of validity if `validity_mode` is `fixed`.
    pub validity_fixed_until: Option<Timestamp>,
    /// Minutes component of the validity duration if `validity_mode` is `dynamic`.
    pub validity_dynamic_duration_minutes: Option<u32>,
    /// Hours component of the validity duration if `validity_mode` is `dynamic`.
    pub validity_dynamic_duration_hours: Option<u32>,
    /// Days component of the validity duration if `validity_mode` is `dynamic`.
    pub validity_dynamic_duration_days: Option<u32>,
    /// Months component of the validity duration if `validity_mode` is `dynamic`.
    pub validity_dynamic_duration_months: Option<u32>,
    /// Whether customers can choose the start of validity (`dynamic` mode).
    pub validity_dynamic_start_choice: bool,
    /// Maximum number of days the chosen start of validity can be in the future.
    pub validity_dynamic_start_choice_day_limit: Option<u32>,
    /// Whether tickets are generated for this item (`None` = regular ticketing rules).
    pub generate_tickets: Option<bool>,
    /// Whether a waiting list is shown when the item is sold out.
    pub allow_waitinglist: bool,
    /// Whether buying this item yields a gift card.
    pub issue_giftcard: bool,
    /// Policy on how to handle reusable media (experimental).
    #[serde(default)]
    pub media_policy: Option<MediaPolicy>,
    /// Type of reusable media to work on (experimental).
    #[serde(default)]
    pub media_type: Option<MediumType>,
    /// Whether to publicly show how many tickets are left (`None` = event default).
    pub show_quota_left: Option<bool>,
    /// Whether this item has variations.
    pub has_variations: bool,
    /// The item's variations.
    #[serde(default)]
    pub variations: Vec<ItemVariation>,
    /// Add-on configurations of this item.
    #[serde(default)]
    pub addons: Vec<ItemAddOn>,
    /// Products bundled with this item.
    #[serde(default)]
    pub bundles: Vec<ItemBundle>,
    /// Program times of this item (not available in event series).
    #[serde(default)]
    pub program_times: Vec<ItemProgramTime>,
    /// Values of item meta data properties.
    #[serde(default)]
    pub meta_data: BTreeMap<String, String>,
}

fn default_hidden_if_item_available_mode() -> AvailabilityMode {
    AvailabilityMode::Hide
}

string_enum! {
    /// How an item or variation is shown while it is unavailable.
    pub enum AvailabilityMode {
        /// Hidden in the shop (default).
        Hide = "hide",
        /// Visible with a note explaining why it cannot be purchased.
        Info = "info",
    }
}

string_enum! {
    /// Special validity behavior of issued tickets.
    pub enum ValidityMode {
        /// Valid in a fixed time frame (`validity_fixed_from`/`validity_fixed_until`).
        Fixed = "fixed",
        /// Valid for a duration from purchase or a chosen start.
        Dynamic = "dynamic",
    }
}

string_enum! {
    /// Policy on how to handle reusable media.
    pub enum MediaPolicy {
        /// Always create a new medium.
        New = "new",
        /// Reuse an existing medium.
        Reuse = "reuse",
        /// Reuse an existing medium or create a new one.
        ReuseOrNew = "reuse_or_new",
        /// Append to an existing medium.
        Append = "append",
        /// Append to an existing medium or create a new one.
        AppendOrNew = "append_or_new",
    }
}

string_enum! {
    /// Fields items can be sorted by.
    pub enum ItemOrdering {
        /// By ID.
        Id = "id",
        /// By position (default).
        Position = "position",
    }
}

/// Query parameters for [`Items::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct ListItems {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<ItemOrdering>>,
    /// Substring search in the name or internal name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub search: Option<String>,
    /// Only items with this `active` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Only items in the category with this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<u64>,
    /// Only items with this `admission` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admission: Option<bool>,
    /// Only items with this tax rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_rate: Option<Decimal>,
    /// Only items with this `free_price` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_price: Option<bool>,
}

/// Body of [`Items::create`].
///
/// An item created with variations always has variations, and vice versa. Variations,
/// add-ons, bundles and program times can only be set here; use the nested endpoints
/// to change them later.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateItem {
    /// The item's visible name.
    #[builder(into)]
    pub name: LocalizedString,
    /// The price applied if not overridden by variations or other options.
    pub default_price: Decimal,
    /// An optional name that is only used in the backend.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub internal_name: Option<String>,
    /// ID of the category this item belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<u64>,
    /// Whether the item is shown in public lists and sold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// A public description (might include Markdown).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub description: Option<LocalizedString>,
    /// Whether customers can choose the price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_price: Option<bool>,
    /// A suggested price, used as a default value if `free_price` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_price_suggestion: Option<Decimal>,
    /// ID of the tax rule to apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_rule: Option<u64>,
    /// Whether the item grants admission to the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admission: Option<bool>,
    /// Whether the item requires personalization (defaults to `admission`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personalized: Option<bool>,
    /// Used for sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// The product picture, uploaded with [`Client::upload`](crate::Client::upload).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<FileId>,
    /// Whether the item is available on all sales channels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_sales_channels: Option<bool>,
    /// Sales channel identifiers the item is available on, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_sales_channels: Option<Vec<String>>,
    /// The first time at which this item can be bought.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_from: Option<Timestamp>,
    /// How the item is shown before `available_from`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub available_from_mode: Option<AvailabilityMode>,
    /// The last time at which this item can be bought.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_until: Option<Timestamp>,
    /// How the item is shown after `available_until`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub available_until_mode: Option<AvailabilityMode>,
    /// **Deprecated**: ID of a quota; the item is hidden as long as that quota is
    /// available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_if_available: Option<u64>,
    /// ID of another item; this item is hidden as long as that item is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_if_item_available: Option<u64>,
    /// How the item is shown while `hidden_if_item_available` is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub hidden_if_item_available_mode: Option<AvailabilityMode>,
    /// Whether the item can only be bought with a voucher assigned to it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_voucher: Option<bool>,
    /// Whether the item is only shown during voucher redemption.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_without_voucher: Option<bool>,
    /// Whether customers can cancel orders containing this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_cancel: Option<bool>,
    /// Minimum number of this item per order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_per_order: Option<u32>,
    /// Maximum number of this item per order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_per_order: Option<u32>,
    /// Whether the check-in app should show a warning when this item is scanned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkin_attention: Option<bool>,
    /// Text shown when a ticket of this item is scanned.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub checkin_text: Option<String>,
    /// An original price, shown for comparison.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_price: Option<Decimal>,
    /// Whether orders with this item need approval by the organizer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<bool>,
    /// Whether this item is only available as part of bundles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_bundling: Option<bool>,
    /// Whether booking this item requires an active membership.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_membership: Option<bool>,
    /// Whether this item is hidden from users without a valid membership.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_membership_hidden: Option<bool>,
    /// IDs of the membership types valid if `require_membership` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_membership_types: Option<Vec<u64>>,
    /// ID of the membership type created when purchasing this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_membership_type: Option<u64>,
    /// Whether the granted membership's term is the purchased (sub-)event's duration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_membership_duration_like_event: Option<bool>,
    /// Days of the granted membership's term.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_membership_duration_days: Option<u32>,
    /// Calendar months of the granted membership's term.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_membership_duration_months: Option<u32>,
    /// Special validity of issued tickets.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub validity_mode: Option<ValidityMode>,
    /// Start of validity if `validity_mode` is `fixed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_fixed_from: Option<Timestamp>,
    /// End of validity if `validity_mode` is `fixed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_fixed_until: Option<Timestamp>,
    /// Minutes component of the validity duration if `validity_mode` is `dynamic`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_dynamic_duration_minutes: Option<u32>,
    /// Hours component of the validity duration if `validity_mode` is `dynamic`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_dynamic_duration_hours: Option<u32>,
    /// Days component of the validity duration if `validity_mode` is `dynamic`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_dynamic_duration_days: Option<u32>,
    /// Months component of the validity duration if `validity_mode` is `dynamic`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_dynamic_duration_months: Option<u32>,
    /// Whether customers can choose the start of validity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_dynamic_start_choice: Option<bool>,
    /// Maximum number of days the chosen start of validity can be in the future.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_dynamic_start_choice_day_limit: Option<u32>,
    /// Whether tickets are generated for this item (unset = regular ticketing rules).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_tickets: Option<bool>,
    /// Whether a waiting list is shown when the item is sold out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_waitinglist: Option<bool>,
    /// Whether buying this item yields a gift card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_giftcard: Option<bool>,
    /// Policy on how to handle reusable media (experimental).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub media_policy: Option<MediaPolicy>,
    /// Type of reusable media to work on (experimental).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub media_type: Option<MediumType>,
    /// Whether to publicly show how many tickets are left (unset = event default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_quota_left: Option<bool>,
    /// Initial variations. An item created without variations can never have any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variations: Option<Vec<CreateItemVariation>>,
    /// Initial add-on configurations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addons: Option<Vec<CreateItemAddOn>>,
    /// Initial bundled products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundles: Option<Vec<CreateItemBundle>>,
    /// Initial program times (not available in event series).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program_times: Option<Vec<CreateItemProgramTime>>,
    /// Values of item meta data properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data: Option<BTreeMap<String, String>>,
}

/// Body of [`Items::update`]. Unset fields are left unchanged.
///
/// Variations, add-ons, bundles and program times cannot be changed here; use the nested
/// endpoints. For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateItem {
    /// The item's visible name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<LocalizedString>,
    /// The price applied if not overridden by variations or other options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_price: Option<Decimal>,
    /// An optional name that is only used in the backend.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub internal_name: Option<Option<String>>,
    /// ID of the category this item belongs to.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub category: Option<Option<u64>>,
    /// Whether the item is shown in public lists and sold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// A public description (might include Markdown).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub description: Option<Option<LocalizedString>>,
    /// Whether customers can choose the price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_price: Option<bool>,
    /// A suggested price, used as a default value if `free_price` is set.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub free_price_suggestion: Option<Option<Decimal>>,
    /// ID of the tax rule to apply.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub tax_rule: Option<Option<u64>>,
    /// Whether the item grants admission to the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admission: Option<bool>,
    /// Whether the item requires personalization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personalized: Option<bool>,
    /// Used for sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// The product picture, uploaded with [`Client::upload`](crate::Client::upload)
    /// (`Some(None)` removes it).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub picture: Option<Option<FileId>>,
    /// Whether the item is available on all sales channels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_sales_channels: Option<bool>,
    /// Sales channel identifiers the item is available on, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_sales_channels: Option<Vec<String>>,
    /// The first time at which this item can be bought.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub available_from: Option<Option<Timestamp>>,
    /// How the item is shown before `available_from`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub available_from_mode: Option<AvailabilityMode>,
    /// The last time at which this item can be bought.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub available_until: Option<Option<Timestamp>>,
    /// How the item is shown after `available_until`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub available_until_mode: Option<AvailabilityMode>,
    /// **Deprecated**: ID of a quota; the item is hidden as long as that quota is
    /// available.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub hidden_if_available: Option<Option<u64>>,
    /// ID of another item; this item is hidden as long as that item is available.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub hidden_if_item_available: Option<Option<u64>>,
    /// How the item is shown while `hidden_if_item_available` is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub hidden_if_item_available_mode: Option<AvailabilityMode>,
    /// Whether the item can only be bought with a voucher assigned to it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_voucher: Option<bool>,
    /// Whether the item is only shown during voucher redemption.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_without_voucher: Option<bool>,
    /// Whether customers can cancel orders containing this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_cancel: Option<bool>,
    /// Minimum number of this item per order.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub min_per_order: Option<Option<u32>>,
    /// Maximum number of this item per order.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub max_per_order: Option<Option<u32>>,
    /// Whether the check-in app should show a warning when this item is scanned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkin_attention: Option<bool>,
    /// Text shown when a ticket of this item is scanned.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub checkin_text: Option<Option<String>>,
    /// An original price, shown for comparison.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub original_price: Option<Option<Decimal>>,
    /// Whether orders with this item need approval by the organizer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<bool>,
    /// Whether this item is only available as part of bundles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_bundling: Option<bool>,
    /// Whether booking this item requires an active membership.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_membership: Option<bool>,
    /// Whether this item is hidden from users without a valid membership.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_membership_hidden: Option<bool>,
    /// IDs of the membership types valid if `require_membership` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_membership_types: Option<Vec<u64>>,
    /// ID of the membership type created when purchasing this item.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub grant_membership_type: Option<Option<u64>>,
    /// Whether the granted membership's term is the purchased (sub-)event's duration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_membership_duration_like_event: Option<bool>,
    /// Days of the granted membership's term.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_membership_duration_days: Option<u32>,
    /// Calendar months of the granted membership's term.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_membership_duration_months: Option<u32>,
    /// Special validity of issued tickets (`Some(None)` = follow event configuration).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub validity_mode: Option<Option<ValidityMode>>,
    /// Start of validity if `validity_mode` is `fixed`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub validity_fixed_from: Option<Option<Timestamp>>,
    /// End of validity if `validity_mode` is `fixed`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub validity_fixed_until: Option<Option<Timestamp>>,
    /// Minutes component of the validity duration if `validity_mode` is `dynamic`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub validity_dynamic_duration_minutes: Option<Option<u32>>,
    /// Hours component of the validity duration if `validity_mode` is `dynamic`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub validity_dynamic_duration_hours: Option<Option<u32>>,
    /// Days component of the validity duration if `validity_mode` is `dynamic`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub validity_dynamic_duration_days: Option<Option<u32>>,
    /// Months component of the validity duration if `validity_mode` is `dynamic`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub validity_dynamic_duration_months: Option<Option<u32>>,
    /// Whether customers can choose the start of validity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_dynamic_start_choice: Option<bool>,
    /// Maximum number of days the chosen start of validity can be in the future.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub validity_dynamic_start_choice_day_limit: Option<Option<u32>>,
    /// Whether tickets are generated for this item (`Some(None)` = regular ticketing
    /// rules).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub generate_tickets: Option<Option<bool>>,
    /// Whether a waiting list is shown when the item is sold out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_waitinglist: Option<bool>,
    /// Whether buying this item yields a gift card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_giftcard: Option<bool>,
    /// Policy on how to handle reusable media (experimental).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub media_policy: Option<Option<MediaPolicy>>,
    /// Type of reusable media to work on (experimental).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub media_type: Option<Option<MediumType>>,
    /// Whether to publicly show how many tickets are left (`Some(None)` = event default).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub show_quota_left: Option<Option<bool>>,
    /// Values of item meta data properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data: Option<BTreeMap<String, String>>,
}

/// Endpoints for the items (products) of an event. Created with [`EventScope::items`].
#[derive(Debug, Clone)]
pub struct Items {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Items (products) of this event.
    #[must_use]
    pub fn items(&self) -> Items {
        Items {
            client: self.client(),
            path: self.path().join("items"),
        }
    }
}

impl Items {
    /// Lists items.
    pub fn list(&self, params: &ListItems) -> Request<Page<Item>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one item.
    pub fn get(&self, id: u64) -> Request<Item> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates an item.
    pub fn create(&self, body: &CreateItem) -> Request<Item> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates an item.
    pub fn update(&self, id: u64, body: &UpdateItem) -> Request<Item> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes an item.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
