//! Campaigns: tracking links for marketing campaigns (plugin `pretix-campaigns`).
//!
//! <https://docs.pretix.eu/dev/api/resources/campaigns.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::{Client, EventScope, ListParams, Page, Request};

/// A marketing campaign with a tracking link.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Campaign {
    /// Internal campaign ID.
    pub id: u64,
    /// The URL component of the campaign (`https://<server>/<organizer>/<event>/c/<code>/`).
    /// Globally unique.
    pub code: String,
    /// An internal, human-readable name of the campaign.
    pub description: String,
    /// URL the tracking link redirects to. Empty to redirect to the ticket shop.
    pub external_target: String,
    /// Number of orders tracked on this campaign.
    pub order_count: u64,
    /// Number of clicks tracked on this campaign.
    pub click_count: u64,
}

/// Body of [`Campaigns::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateCampaign {
    /// An internal, human-readable name of the campaign.
    #[builder(into)]
    pub description: String,
    /// The URL component of the campaign. Must be globally unique; a random value is
    /// chosen if omitted (recommended).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub code: Option<String>,
    /// URL the tracking link redirects to. Empty to redirect to the ticket shop.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub external_target: Option<String>,
}

/// Body of [`Campaigns::update`]. Unset fields are left unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateCampaign {
    /// The URL component of the campaign. Must be globally unique.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub code: Option<String>,
    /// An internal, human-readable name of the campaign.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub description: Option<String>,
    /// URL the tracking link redirects to. Empty to redirect to the ticket shop.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub external_target: Option<String>,
}

/// Endpoints for the campaigns of an event. Created with [`EventScope::campaigns`].
#[derive(Debug, Clone)]
pub struct Campaigns {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Campaigns of this event.
    #[must_use]
    pub fn campaigns(&self) -> Campaigns {
        Campaigns {
            client: self.client(),
            path: self.path().join("campaigns"),
        }
    }
}

impl Campaigns {
    /// Lists campaigns.
    pub fn list(&self, params: &ListParams) -> Request<Page<Campaign>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one campaign.
    pub fn get(&self, id: u64) -> Request<Campaign> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a campaign.
    pub fn create(&self, body: &CreateCampaign) -> Request<Campaign> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a campaign.
    pub fn update(&self, id: u64, body: &UpdateCampaign) -> Request<Campaign> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a campaign and all associated data.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
