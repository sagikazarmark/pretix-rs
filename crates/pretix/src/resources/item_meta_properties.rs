//! Item meta properties: event-internal meta information fields for items (products).
//!
//! <https://docs.pretix.eu/dev/api/resources/item_meta_properties.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::{Client, EventScope, ListParams, Page, Request};

/// An item meta property.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ItemMetaProperty {
    /// Unique ID for this property.
    pub id: u64,
    /// Name of the property.
    pub name: String,
    /// Default value.
    pub default: String,
    /// Whether all items of the event need a value for this property.
    pub required: bool,
    /// Permitted values, or `None` for no limitation.
    #[serde(default)]
    pub allowed_values: Option<Vec<String>>,
}

/// Body of [`ItemMetaProperties::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateItemMetaProperty {
    /// Name of the property.
    #[builder(into)]
    pub name: String,
    /// Default value.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub default: Option<String>,
    /// Whether all items of the event need a value for this property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// Permitted values; `None` (sent as `null`) for no limitation. pretix rejects an
    /// empty list.
    pub allowed_values: Option<Vec<String>>,
}

/// Body of [`ItemMetaProperties::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateItemMetaProperty {
    /// Name of the property.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<String>,
    /// Default value.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub default: Option<String>,
    /// Whether all items of the event need a value for this property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// Permitted values (`Some(None)` removes the limitation). pretix rejects an empty
    /// list.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub allowed_values: Option<Option<Vec<String>>>,
}

/// Endpoints for the item meta properties of an event. Created with
/// [`EventScope::item_meta_properties`].
#[derive(Debug, Clone)]
pub struct ItemMetaProperties {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Item meta properties of this event.
    #[must_use]
    pub fn item_meta_properties(&self) -> ItemMetaProperties {
        ItemMetaProperties {
            client: self.client(),
            path: self.path().join("item_meta_properties"),
        }
    }
}

impl ItemMetaProperties {
    /// Lists item meta properties.
    pub fn list(&self, params: &ListParams) -> Request<Page<ItemMetaProperty>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one item meta property.
    pub fn get(&self, id: u64) -> Request<ItemMetaProperty> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates an item meta property.
    pub fn create(&self, body: &CreateItemMetaProperty) -> Request<ItemMetaProperty> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates an item meta property.
    pub fn update(&self, id: u64, body: &UpdateItemMetaProperty) -> Request<ItemMetaProperty> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes an item meta property.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
