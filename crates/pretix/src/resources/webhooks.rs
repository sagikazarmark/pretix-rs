//! Webhooks: URLs that pretix notifies about changes.
//!
//! This module manages webhook configuration. The body pretix sends to a webhook is
//! [`WebhookNotification`](crate::types::WebhookNotification).
//!
//! <https://docs.pretix.eu/dev/api/resources/webhooks.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::{Client, OrganizerScope, Page, Request};

/// A webhook.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Webhook {
    /// Internal ID of the webhook.
    pub id: u64,
    /// If `false`, this webhook does not receive any notifications.
    pub enabled: bool,
    /// The URL to call.
    pub target_url: String,
    /// Whether this webhook receives notifications for all events of the organizer.
    pub all_events: bool,
    /// Slugs of the events this webhook is active for, if not all.
    #[serde(default)]
    pub limit_events: Vec<String>,
    /// Action types that are sent to this webhook, e.g. `pretix.event.order.paid`.
    /// Some types end with a wildcard, e.g. `pretix.event.order.changed.*`.
    #[serde(default)]
    pub action_types: Vec<String>,
    /// Internal comment.
    pub comment: Option<String>,
}

/// Query parameters for [`Webhooks::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct ListWebhooks {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Only webhooks that are (`true`) or are not (`false`) enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// Body of [`Webhooks::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateWebhook {
    /// The URL to call.
    #[builder(into)]
    pub target_url: String,
    /// Action types to send to this webhook.
    pub action_types: Vec<String>,
    /// If `false`, this webhook does not receive any notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Whether this webhook receives notifications for all events of the organizer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_events: Option<bool>,
    /// Slugs of the events this webhook is active for, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_events: Option<Vec<String>>,
    /// Internal comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub comment: Option<String>,
}

/// Body of [`Webhooks::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateWebhook {
    /// If `false`, this webhook does not receive any notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// The URL to call.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub target_url: Option<String>,
    /// Whether this webhook receives notifications for all events of the organizer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_events: Option<bool>,
    /// Slugs of the events this webhook is active for, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_events: Option<Vec<String>>,
    /// Action types to send to this webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_types: Option<Vec<String>>,
    /// Internal comment.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub comment: Option<Option<String>>,
}

/// Endpoints for the webhooks of an organizer. Created with [`OrganizerScope::webhooks`].
#[derive(Debug, Clone)]
pub struct Webhooks {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Webhooks of this organizer.
    #[must_use]
    pub fn webhooks(&self) -> Webhooks {
        Webhooks {
            client: self.client(),
            path: self.path().join("webhooks"),
        }
    }
}

impl Webhooks {
    /// Lists webhooks.
    pub fn list(&self, params: &ListWebhooks) -> Request<Page<Webhook>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one webhook.
    pub fn get(&self, id: u64) -> Request<Webhook> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a webhook.
    pub fn create(&self, body: &CreateWebhook) -> Request<Webhook> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a webhook.
    pub fn update(&self, id: u64, body: &UpdateWebhook) -> Request<Webhook> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a webhook. pretix currently only disables it.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
