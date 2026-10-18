use serde::{Deserialize, Serialize};

use super::Items;
use crate::path::Path;
use crate::{Client, ListParams, Page, Request};

/// Add-on configuration: a category of products that can be added to an item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ItemAddOn {
    /// Internal ID of the add-on. Not included when embedded in an
    /// [`Item`](super::Item).
    #[serde(default)]
    pub id: Option<u64>,
    /// ID of the item category the add-ons can be chosen from.
    pub addon_category: u64,
    /// Minimal number of add-ons that need to be chosen.
    pub min_count: u32,
    /// Maximal number of add-ons that can be chosen.
    pub max_count: u32,
    /// Used for sorting.
    pub position: i64,
    /// Whether the same item can be added multiple times.
    pub multi_allowed: bool,
    /// Whether adding these add-ons is free.
    pub price_included: bool,
}

/// Body of [`ItemAddOns::create`], also used for the initial add-ons in
/// [`CreateItem`](super::CreateItem).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateItemAddOn {
    /// ID of the item category the add-ons can be chosen from.
    pub addon_category: u64,
    /// Minimal number of add-ons that need to be chosen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_count: Option<u32>,
    /// Maximal number of add-ons that can be chosen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_count: Option<u32>,
    /// Used for sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// Whether the same item can be added multiple times.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_allowed: Option<bool>,
    /// Whether adding these add-ons is free.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_included: Option<bool>,
}

/// Body of [`ItemAddOns::update`]. Unset fields are left unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateItemAddOn {
    /// ID of the item category the add-ons can be chosen from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addon_category: Option<u64>,
    /// Minimal number of add-ons that need to be chosen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_count: Option<u32>,
    /// Maximal number of add-ons that can be chosen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_count: Option<u32>,
    /// Used for sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// Whether the same item can be added multiple times.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_allowed: Option<bool>,
    /// Whether adding these add-ons is free.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_included: Option<bool>,
}

/// Endpoints for the add-on configurations of an item. Created with [`Items::addons`].
#[derive(Debug, Clone)]
pub struct ItemAddOns {
    client: Client,
    path: Path,
}

impl Items {
    /// Add-on configurations of the item with ID `item`.
    #[must_use]
    pub fn addons(&self, item: u64) -> ItemAddOns {
        ItemAddOns {
            client: self.client.clone(),
            path: self.path.join(item).join("addons"),
        }
    }
}

impl ItemAddOns {
    /// Lists add-ons.
    pub fn list(&self, params: &ListParams) -> Request<Page<ItemAddOn>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one add-on.
    pub fn get(&self, id: u64) -> Request<ItemAddOn> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates an add-on.
    pub fn create(&self, body: &CreateItemAddOn) -> Request<ItemAddOn> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates an add-on.
    pub fn update(&self, id: u64, body: &UpdateItemAddOn) -> Request<ItemAddOn> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes an add-on.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
