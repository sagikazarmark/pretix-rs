//! Waiting list entries of an event.
//!
//! <https://docs.pretix.eu/dev/api/resources/waitinglist.html>

use std::collections::BTreeMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Ordering;
use crate::types::string_enum;
use crate::{Client, EventScope, Page, Request};

/// An entry on the waiting list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WaitingListEntry {
    /// Internal ID of the entry.
    pub id: u64,
    /// When the entry was created.
    pub created: Timestamp,
    /// Name of the waiting person.
    #[serde(default)]
    pub name: Option<String>,
    /// Decomposition of the name of the waiting person.
    #[serde(default)]
    pub name_parts: Option<BTreeMap<String, String>>,
    /// Email address of the waiting person.
    pub email: String,
    /// Phone number of the waiting person.
    #[serde(default)]
    pub phone: Option<String>,
    /// ID of the voucher sent to the person. `None` while the person is still waiting.
    pub voucher: Option<u64>,
    /// ID of the item the person is waiting for.
    pub item: u64,
    /// ID of the variation the person is waiting for.
    pub variation: Option<u64>,
    /// Locale of the waiting person.
    pub locale: String,
    /// ID of the date inside an event series this entry belongs to.
    pub subevent: Option<u64>,
    /// Priority of the entry; entries with a higher priority receive vouchers first
    /// (not in the pretix docs).
    #[serde(default)]
    pub priority: i64,
}

string_enum! {
    /// Fields waiting list entries can be sorted by.
    pub enum WaitingListEntryOrdering {
        /// By ID.
        Id = "id",
        /// By creation date (default).
        Created = "created",
        /// By email address.
        Email = "email",
        /// By item.
        Item = "item",
    }
}

/// Query parameters for [`WaitingListEntries::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct ListWaitingListEntries {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<WaitingListEntryOrdering>>,
    /// Only entries with this email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub email: Option<String>,
    /// Only entries with this locale.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub locale: Option<String>,
    /// Only entries that have (`true`) or have not (`false`) been sent a voucher.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_voucher: Option<bool>,
    /// Only entries waiting for this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<u64>,
    /// Only entries waiting for this variation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// Only entries of this sub-event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
}

/// Body of [`WaitingListEntries::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateWaitingListEntry {
    /// Email address of the waiting person.
    #[builder(into)]
    pub email: String,
    /// ID of the item the person is waiting for.
    pub item: u64,
    /// Name of the waiting person.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<String>,
    /// Decomposition of the name of the waiting person.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_parts: Option<BTreeMap<String, String>>,
    /// Phone number of the waiting person.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub phone: Option<String>,
    /// ID of the variation the person is waiting for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// Locale of the waiting person.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub locale: Option<String>,
    /// ID of the date inside an event series this entry belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// Priority of the entry; entries with a higher priority receive vouchers first
    /// (not in the pretix docs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
}

/// Body of [`WaitingListEntries::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateWaitingListEntry {
    /// Email address of the waiting person.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub email: Option<String>,
    /// ID of the item the person is waiting for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<u64>,
    /// Name of the waiting person.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub name: Option<Option<String>>,
    /// Decomposition of the name of the waiting person.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub name_parts: Option<Option<BTreeMap<String, String>>>,
    /// Phone number of the waiting person.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub phone: Option<Option<String>>,
    /// ID of the variation the person is waiting for.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub variation: Option<Option<u64>>,
    /// Locale of the waiting person.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub locale: Option<String>,
    /// ID of the date inside an event series this entry belongs to.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub subevent: Option<Option<u64>>,
    /// Priority of the entry; entries with a higher priority receive vouchers first
    /// (not in the pretix docs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
}

/// Endpoints for the waiting list of an event. Created with
/// [`EventScope::waiting_list_entries`].
#[derive(Debug, Clone)]
pub struct WaitingListEntries {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Waiting list entries of this event.
    #[must_use]
    pub fn waiting_list_entries(&self) -> WaitingListEntries {
        WaitingListEntries {
            client: self.client(),
            path: self.path().join("waitinglistentries"),
        }
    }
}

impl WaitingListEntries {
    /// Lists waiting list entries.
    pub fn list(&self, params: &ListWaitingListEntries) -> Request<Page<WaitingListEntry>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one entry.
    pub fn get(&self, id: u64) -> Request<WaitingListEntry> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates an entry.
    pub fn create(&self, body: &CreateWaitingListEntry) -> Request<WaitingListEntry> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates an entry. Only possible as long as no voucher has been sent.
    pub fn update(&self, id: u64, body: &UpdateWaitingListEntry) -> Request<WaitingListEntry> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Sends a voucher to the waiting person.
    pub fn send_voucher(&self, id: u64) -> Request<()> {
        self.client
            .post(&self.path.join(id).join("send_voucher"))
            .empty()
    }

    /// Deletes an entry. Not possible once a voucher has been sent.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
