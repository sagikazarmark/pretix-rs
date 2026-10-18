use serde::{Deserialize, Serialize};

use super::Items;
use crate::path::Path;
use crate::types::Decimal;
use crate::{Client, ListParams, Page, Request};

/// A product included in another product (bundle configuration).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ItemBundle {
    /// Internal ID of the bundle configuration. Not included when embedded in an
    /// [`Item`](super::Item).
    #[serde(default)]
    pub id: Option<u64>,
    /// ID of the included item.
    pub bundled_item: u64,
    /// ID of the variation of the included item.
    pub bundled_variation: Option<u64>,
    /// Number of items included.
    pub count: u32,
    /// Designated price of the bundled product, used to split the price of the base item
    /// (e.g. for mixed taxation). Not added to the price.
    pub designated_price: Decimal,
}

/// Body of [`ItemBundles::create`], also used for the initial bundles in
/// [`CreateItem`](super::CreateItem).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateItemBundle {
    /// ID of the included item.
    pub bundled_item: u64,
    /// ID of the variation of the included item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundled_variation: Option<u64>,
    /// Number of items included (default 1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    /// Designated price of the bundled product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub designated_price: Option<Decimal>,
}

/// Body of [`ItemBundles::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateItemBundle {
    /// ID of the included item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundled_item: Option<u64>,
    /// ID of the variation of the included item.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub bundled_variation: Option<Option<u64>>,
    /// Number of items included.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    /// Designated price of the bundled product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub designated_price: Option<Decimal>,
}

/// Endpoints for the bundle configurations of an item. Created with [`Items::bundles`].
#[derive(Debug, Clone)]
pub struct ItemBundles {
    client: Client,
    path: Path,
}

impl Items {
    /// Bundle configurations of the item with ID `item`.
    #[must_use]
    pub fn bundles(&self, item: u64) -> ItemBundles {
        ItemBundles {
            client: self.client.clone(),
            path: self.path.join(item).join("bundles"),
        }
    }
}

impl ItemBundles {
    /// Lists bundle configurations.
    pub fn list(&self, params: &ListParams) -> Request<Page<ItemBundle>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one bundle configuration.
    pub fn get(&self, id: u64) -> Request<ItemBundle> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a bundle configuration.
    pub fn create(&self, body: &CreateItemBundle) -> Request<ItemBundle> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a bundle configuration.
    pub fn update(&self, id: u64, body: &UpdateItemBundle) -> Request<ItemBundle> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a bundle configuration.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
