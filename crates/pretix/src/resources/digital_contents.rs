//! Digital content shown to ticket buyers, such as live streams, videos or downloads
//! (plugin `pretix-digital`).
//!
//! <https://docs.pretix.eu/dev/api/resources/digital.html>

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::types::{FileId, LocalizedString, string_enum};
use crate::{Client, EventScope, ListParams, Page, Request};

/// A piece of digital content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct DigitalContent {
    /// Internal content ID.
    pub id: u64,
    /// The content title.
    pub title: LocalizedString,
    /// An optional name that is only used in the backend.
    #[serde(default)]
    pub internal_name: Option<String>,
    /// The type of content.
    pub content_type: DigitalContentType,
    /// The location of the digital content. URLs may contain `{variable}` placeholders.
    pub url: Option<String>,
    /// URL of a downloadable file. Either `url` or `file` is `None`.
    pub file: Option<String>,
    /// A public description (may contain Markdown).
    pub description: Option<LocalizedString>,
    /// The first time at which this content is shown.
    pub available_from: Option<Timestamp>,
    /// The last time at which this content is shown.
    pub available_until: Option<Timestamp>,
    /// Whether the content is available to all buyers of tickets for this event
    /// (`limit_products` is ignored then).
    pub all_products: bool,
    /// Product IDs this content is shown to.
    #[serde(default)]
    pub limit_products: Vec<u64>,
    /// Used for sorting.
    pub position: i64,
    /// Date in an event series this content is shown for (`None` for all).
    pub subevent: Option<u64>,
    /// Template for JWT generation.
    #[serde(default)]
    pub jwt_template: Option<String>,
    /// Secret for JWT generation (`HS256`).
    #[serde(default)]
    pub jwt_secret: Option<String>,
    /// JWT validity in days.
    #[serde(default)]
    pub jwt_validity: Option<u32>,
}

string_enum! {
    /// The type of a [`DigitalContent`].
    pub enum DigitalContentType {
        /// A webinar.
        Webinar = "webinar",
        /// A video.
        Video = "video",
        /// A live stream.
        Livestream = "livestream",
        /// A link.
        Link = "link",
        /// A downloadable file.
        File = "file",
    }
}

/// Body of [`DigitalContents::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateDigitalContent {
    /// The content title.
    #[builder(into)]
    pub title: LocalizedString,
    /// The type of content.
    pub content_type: DigitalContentType,
    /// An optional name that is only used in the backend.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub internal_name: Option<String>,
    /// The location of the digital content. Either `url` or `file` must be unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub url: Option<String>,
    /// A downloadable file (see [`Client::upload`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FileId>,
    /// A public description (may contain Markdown).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub description: Option<LocalizedString>,
    /// The first time at which this content is shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_from: Option<Timestamp>,
    /// The last time at which this content is shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_until: Option<Timestamp>,
    /// Whether the content is available to all buyers of tickets for this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_products: Option<bool>,
    /// Product IDs this content is shown to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_products: Option<Vec<u64>>,
    /// Used for sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// Date in an event series this content is shown for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// Template for JWT generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub jwt_template: Option<String>,
    /// Secret for JWT generation (`HS256`).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub jwt_secret: Option<String>,
    /// JWT validity in days.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt_validity: Option<u32>,
}

/// Body of [`DigitalContents::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateDigitalContent {
    /// The content title.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub title: Option<LocalizedString>,
    /// The type of content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<DigitalContentType>,
    /// An optional name that is only used in the backend.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub internal_name: Option<Option<String>>,
    /// The location of the digital content.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub url: Option<Option<String>>,
    /// A downloadable file (see [`Client::upload`]).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub file: Option<Option<FileId>>,
    /// A public description (may contain Markdown).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub description: Option<Option<LocalizedString>>,
    /// The first time at which this content is shown.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub available_from: Option<Option<Timestamp>>,
    /// The last time at which this content is shown.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub available_until: Option<Option<Timestamp>>,
    /// Whether the content is available to all buyers of tickets for this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_products: Option<bool>,
    /// Product IDs this content is shown to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_products: Option<Vec<u64>>,
    /// Used for sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// Date in an event series this content is shown for.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub subevent: Option<Option<u64>>,
    /// Template for JWT generation.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub jwt_template: Option<Option<String>>,
    /// Secret for JWT generation (`HS256`).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub jwt_secret: Option<Option<String>>,
    /// JWT validity in days.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub jwt_validity: Option<Option<u32>>,
}

/// Endpoints for the digital content of an event. Created with
/// [`EventScope::digital_contents`].
#[derive(Debug, Clone)]
pub struct DigitalContents {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Digital content of this event.
    #[must_use]
    pub fn digital_contents(&self) -> DigitalContents {
        DigitalContents {
            client: self.client(),
            path: self.path().join("digitalcontents"),
        }
    }
}

impl DigitalContents {
    /// Lists digital content.
    pub fn list(&self, params: &ListParams) -> Request<Page<DigitalContent>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one piece of digital content.
    pub fn get(&self, id: u64) -> Request<DigitalContent> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates digital content.
    pub fn create(&self, body: &CreateDigitalContent) -> Request<DigitalContent> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates digital content.
    pub fn update(&self, id: u64, body: &UpdateDigitalContent) -> Request<DigitalContent> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes digital content.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
