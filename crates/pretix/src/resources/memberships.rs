//! Memberships: customer memberships of a membership type, valid for a time span.
//!
//! <https://docs.pretix.eu/dev/api/resources/memberships.html>

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::{Client, OrganizerScope, Page, Request};

/// A membership.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct Membership {
    /// Internal ID of the membership.
    pub id: u64,
    /// Identifier of the customer. Cannot be changed.
    pub customer: String,
    /// Whether this is a test membership.
    pub testmode: bool,
    /// Internal ID of the membership type.
    pub membership_type: u64,
    /// Start of validity.
    pub date_start: Timestamp,
    /// End of validity.
    pub date_end: Timestamp,
    /// Components of the attendee name (configuration dependent).
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub attendee_name_parts: serde_json::Map<String, serde_json::Value>,
}

/// Query parameters for [`Memberships::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListMemberships {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Only memberships of the customer with this identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub customer: Option<String>,
    /// Only memberships of this type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub membership_type: Option<u64>,
    /// Only memberships (not) in test mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testmode: Option<bool>,
}

/// Body of [`Memberships::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateMembership {
    /// Internal ID of the membership type.
    pub membership_type: u64,
    /// Identifier of the customer.
    #[builder(into)]
    pub customer: String,
    /// Whether this is a test membership.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testmode: Option<bool>,
    /// Start of validity.
    pub date_start: Timestamp,
    /// End of validity.
    pub date_end: Timestamp,
    /// Components of the attendee name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attendee_name_parts: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Body of [`Memberships::update`]. Unset fields are left unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateMembership {
    /// Internal ID of the membership type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub membership_type: Option<u64>,
    /// Start of validity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_start: Option<Timestamp>,
    /// End of validity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_end: Option<Timestamp>,
    /// Components of the attendee name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attendee_name_parts: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Endpoints for the memberships of an organizer's customers. Created with
/// [`OrganizerScope::memberships`].
#[derive(Debug, Clone)]
pub struct Memberships {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Customer memberships of this organizer.
    #[must_use]
    pub fn memberships(&self) -> Memberships {
        Memberships {
            client: self.client(),
            path: self.path().join("memberships"),
        }
    }
}

impl Memberships {
    /// Lists memberships.
    pub fn list(&self, params: &ListMemberships) -> Request<Page<Membership>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one membership.
    pub fn get(&self, id: u64) -> Request<Membership> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a membership.
    pub fn create(&self, body: &CreateMembership) -> Request<Membership> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a membership.
    pub fn update(&self, id: u64, body: &UpdateMembership) -> Request<Membership> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }
}
