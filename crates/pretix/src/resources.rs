//! API resources, one module per resource type.
//!
//! Every module follows the same structure:
//!
//! - response models (e.g. `Order`) and enums for enum-like fields,
//! - query parameters for list endpoints (`ListOrders`),
//! - request bodies (`CreateOrder`, `UpdateOrder`, action bodies like `MarkPaid`),
//! - a handle (`Orders`) with one method per endpoint, reached from the client or a
//!   scope (e.g. [`EventScope`](crate::EventScope)`::orders()`).

pub mod auto_checkin_rules;
pub mod badges;
pub mod bank_transfer;
pub mod carts;
pub mod categories;
pub mod checkin;
pub mod checkin_lists;
pub mod customers;
pub mod devices;
pub mod discounts;
pub mod event_meta_properties;
pub mod events;
pub mod exporters;
pub mod gift_cards;
pub mod invoices;
pub mod item_meta_properties;
pub mod items;
pub mod membership_types;
pub mod memberships;
pub mod orders;
pub mod organizers;
pub mod questions;
pub mod quotas;
pub mod reusable_media;
pub mod sales_channels;
pub mod scheduled_exports;
pub mod seating_plans;
pub mod seats;
pub mod sendmail_rules;
pub mod shredders;
pub mod subevents;
pub mod tax_rules;
pub mod teams;
pub mod ticket_layouts;
pub mod transactions;
pub mod vouchers;
pub mod waiting_list;
pub mod webhooks;

#[cfg(feature = "plugins")]
pub mod campaigns;
#[cfg(feature = "plugins")]
pub mod certificates;
#[cfg(feature = "plugins")]
pub mod digital_contents;
#[cfg(feature = "plugins")]
pub mod exhibitors;
#[cfg(feature = "plugins")]
pub mod imported_secrets;
#[cfg(feature = "plugins")]
pub mod offline_sales;
#[cfg(feature = "plugins")]
pub mod shipping;

#[cfg(feature = "hosted")]
pub mod billing_invoices;
#[cfg(feature = "hosted")]
pub mod resellers;

/// Settings of an organizer or event: a dynamic key-value map.
///
/// pretix does not guarantee backwards compatibility for settings, so they are not
/// modeled. Use [`Request::decode_as`](crate::Request::decode_as) to decode into a
/// custom type.
pub type Settings = serde_json::Map<String, serde_json::Value>;

/// Query parameters of settings endpoints.
#[derive(serde::Serialize)]
pub(crate) struct SettingsQuery {
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub(crate) explain: bool,
}
