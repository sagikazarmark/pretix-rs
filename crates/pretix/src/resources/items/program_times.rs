use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::Items;
use crate::path::Path;
use crate::types::LocalizedString;
use crate::{Client, ListParams, Page, Request};

/// A program time slot of an item, e.g. to display separate schedules within an event.
/// Not available for items in event series.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ItemProgramTime {
    /// Internal ID of the program time. Not included when embedded in an
    /// [`Item`](super::Item).
    #[serde(default)]
    pub id: Option<u64>,
    /// Start of the time slot.
    pub start: Timestamp,
    /// End of the time slot.
    pub end: Timestamp,
    /// Location of the time slot.
    #[serde(default)]
    pub location: Option<LocalizedString>,
}

/// Body of [`ItemProgramTimes::create`], also used for the initial program times in
/// [`CreateItem`](super::CreateItem).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateItemProgramTime {
    /// Start of the time slot.
    pub start: Timestamp,
    /// End of the time slot.
    pub end: Timestamp,
    /// Location of the time slot.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub location: Option<LocalizedString>,
}

/// Body of [`ItemProgramTimes::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateItemProgramTime {
    /// Start of the time slot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<Timestamp>,
    /// End of the time slot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<Timestamp>,
    /// Location of the time slot.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub location: Option<Option<LocalizedString>>,
}

/// Endpoints for the program times of an item. Created with [`Items::program_times`].
#[derive(Debug, Clone)]
pub struct ItemProgramTimes {
    client: Client,
    path: Path,
}

impl Items {
    /// Program times of the item with ID `item`.
    #[must_use]
    pub fn program_times(&self, item: u64) -> ItemProgramTimes {
        ItemProgramTimes {
            client: self.client.clone(),
            path: self.path.join(item).join("program_times"),
        }
    }
}

impl ItemProgramTimes {
    /// Lists program times.
    pub fn list(&self, params: &ListParams) -> Request<Page<ItemProgramTime>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one program time.
    pub fn get(&self, id: u64) -> Request<ItemProgramTime> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a program time.
    pub fn create(&self, body: &CreateItemProgramTime) -> Request<ItemProgramTime> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a program time.
    pub fn update(&self, id: u64, body: &UpdateItemProgramTime) -> Request<ItemProgramTime> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a program time.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
