//! Sales channels of an organizer.
//!
//! <https://docs.pretix.eu/dev/api/resources/saleschannels.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::types::{LocalizedString, string_enum};
use crate::{Client, ListParams, OrganizerScope, Page, Request};

/// A sales channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SalesChannel {
    /// Identifier of the sales channel. Equals `type` for types that allow only one
    /// instance, otherwise it is prefixed with `type.`.
    pub identifier: String,
    /// Human-readable name.
    pub label: LocalizedString,
    /// Type of the sales channel.
    pub r#type: SalesChannelType,
    /// Used for sorting.
    pub position: i64,
}

string_enum! {
    /// The type of a sales channel.
    pub enum SalesChannelType {
        /// The online shop.
        Web = "web",
        /// A custom API integration (the only type that can be created through the API).
        Api = "api",
    }
}

/// Body of [`SalesChannels::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateSalesChannel {
    /// Identifier of the sales channel, e.g. `api.custom`.
    #[builder(into)]
    pub identifier: String,
    /// Human-readable name.
    #[builder(into)]
    pub label: LocalizedString,
    /// Type of the sales channel. Only [`SalesChannelType::Api`] can be created, which
    /// is also the default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<SalesChannelType>,
    /// Used for sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
}

/// Body of [`SalesChannels::update`]. Unset fields are left unchanged.
///
/// `identifier` and `type` cannot be changed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateSalesChannel {
    /// Human-readable name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub label: Option<LocalizedString>,
    /// Used for sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
}

/// Endpoints for the sales channels of an organizer. Created with
/// [`OrganizerScope::sales_channels`].
#[derive(Debug, Clone)]
pub struct SalesChannels {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Sales channels of this organizer.
    #[must_use]
    pub fn sales_channels(&self) -> SalesChannels {
        SalesChannels {
            client: self.client(),
            path: self.path().join("saleschannels"),
        }
    }
}

impl SalesChannels {
    /// Lists sales channels.
    pub fn list(&self, params: &ListParams) -> Request<Page<SalesChannel>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one sales channel.
    pub fn get(&self, identifier: &str) -> Request<SalesChannel> {
        self.client.get(&self.path.join(identifier)).json()
    }

    /// Creates a sales channel.
    pub fn create(&self, body: &CreateSalesChannel) -> Request<SalesChannel> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a sales channel.
    pub fn update(&self, identifier: &str, body: &UpdateSalesChannel) -> Request<SalesChannel> {
        self.client
            .patch(&self.path.join(identifier))
            .json_body(body)
            .json()
    }

    /// Deletes a sales channel. Channels that have been used or are integral parts of
    /// the system cannot be deleted.
    pub fn delete(&self, identifier: &str) -> Request<()> {
        self.client.delete(&self.path.join(identifier)).empty()
    }
}
