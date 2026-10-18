//! Event meta properties: organizer-defined meta information fields for events.
//!
//! <https://docs.pretix.eu/dev/api/resources/event_meta_properties.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::types::LocalizedString;
use crate::{Client, ListParams, OrganizerScope, Page, Request};

/// An event meta property.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct EventMetaProperty {
    /// Unique ID for this property.
    pub id: u64,
    /// Name of the property.
    pub name: String,
    /// Default value.
    pub default: String,
    /// Whether an event can only be taken live if the property is set (always optional
    /// for dates of event series).
    pub required: bool,
    /// Whether the value can only be changed by organizer-level administrators.
    pub protected: bool,
    /// Whether the property is shown to filter events in the public event list and
    /// calendar.
    pub filter_public: bool,
    /// Public name of the property.
    pub public_label: Option<LocalizedString>,
    /// Whether the property can be used to filter events or reports in the backend (and
    /// for hidden filter parameters in the frontend).
    pub filter_allowed: bool,
    /// Permitted values, or `None` for no limitation.
    #[serde(default)]
    pub choices: Option<Vec<EventMetaPropertyChoice>>,
}

/// A permitted value of an [`EventMetaProperty`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct EventMetaPropertyChoice {
    /// Internal name of the value.
    #[builder(into)]
    pub key: String,
    /// Public name of the value.
    // Also used in request bodies, where pretix expects the key to be omitted rather than
    // `null`; responses omit it as well when no label is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub label: Option<LocalizedString>,
}

/// Body of [`EventMetaProperties::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateEventMetaProperty {
    /// Name of the property.
    #[builder(into)]
    pub name: String,
    /// Default value.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub default: Option<String>,
    /// Whether an event can only be taken live if the property is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// Whether the value can only be changed by organizer-level administrators.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protected: Option<bool>,
    /// Whether the property is shown to filter events in the public event list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_public: Option<bool>,
    /// Public name of the property.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub public_label: Option<LocalizedString>,
    /// Whether the property can be used to filter events or reports in the backend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_allowed: Option<bool>,
    /// Permitted values; `None` (sent as `null`) for no limitation.
    pub choices: Option<Vec<EventMetaPropertyChoice>>,
}

/// Body of [`EventMetaProperties::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateEventMetaProperty {
    /// Name of the property.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<String>,
    /// Default value.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub default: Option<String>,
    /// Whether an event can only be taken live if the property is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// Whether the value can only be changed by organizer-level administrators.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protected: Option<bool>,
    /// Whether the property is shown to filter events in the public event list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_public: Option<bool>,
    /// Public name of the property.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub public_label: Option<Option<LocalizedString>>,
    /// Whether the property can be used to filter events or reports in the backend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_allowed: Option<bool>,
    /// Permitted values (`Some(None)` removes the limitation).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub choices: Option<Option<Vec<EventMetaPropertyChoice>>>,
}

/// Endpoints for the event meta properties of an organizer. Created with
/// [`OrganizerScope::event_meta_properties`].
#[derive(Debug, Clone)]
pub struct EventMetaProperties {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Event meta properties of this organizer.
    #[must_use]
    pub fn event_meta_properties(&self) -> EventMetaProperties {
        EventMetaProperties {
            client: self.client(),
            path: self.path().join("event_meta_properties"),
        }
    }
}

impl EventMetaProperties {
    /// Lists event meta properties.
    pub fn list(&self, params: &ListParams) -> Request<Page<EventMetaProperty>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one event meta property.
    pub fn get(&self, id: u64) -> Request<EventMetaProperty> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates an event meta property.
    pub fn create(&self, body: &CreateEventMetaProperty) -> Request<EventMetaProperty> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates an event meta property.
    pub fn update(&self, id: u64, body: &UpdateEventMetaProperty) -> Request<EventMetaProperty> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes an event meta property.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
