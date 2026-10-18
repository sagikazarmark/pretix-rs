//! Integration tests against a mock pretix server.

#![allow(unreachable_pub, missing_docs, clippy::missing_panics_doc)]

mod support;

mod core;

mod auto_checkin_rules;
mod badges;
mod bank_transfer;
mod carts;
mod categories;
mod checkin;
mod checkin_lists;
mod customers;
mod devices;
mod discounts;
mod event_meta_properties;
mod events;
mod exporters;
mod gift_cards;
mod invoices;
mod item_meta_properties;
mod items;
mod membership_types;
mod memberships;
mod orders;
mod organizers;
mod questions;
mod quotas;
mod reusable_media;
mod sales_channels;
mod scheduled_exports;
mod seating_plans;
mod seats;
mod sendmail_rules;
mod shredders;
mod subevents;
mod tax_rules;
mod teams;
mod ticket_layouts;
mod transactions;
mod vouchers;
mod waiting_list;
mod webhooks;

#[cfg(feature = "plugins")]
mod campaigns;
#[cfg(feature = "plugins")]
mod certificates;
#[cfg(feature = "plugins")]
mod digital_contents;
#[cfg(feature = "plugins")]
mod exhibitors;
#[cfg(feature = "plugins")]
mod imported_secrets;
#[cfg(feature = "plugins")]
mod offline_sales;
#[cfg(feature = "plugins")]
mod shipping;

#[cfg(feature = "hosted")]
mod billing_invoices;
#[cfg(feature = "hosted")]
mod resellers;
