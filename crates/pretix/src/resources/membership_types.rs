//! Membership types: kinds of memberships customers can hold (e.g. a week pass).
//!
//! <https://docs.pretix.eu/dev/api/resources/membershiptypes.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::types::LocalizedString;
use crate::{Client, ListParams, OrganizerScope, Page, Request};

/// A membership type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MembershipType {
    /// Internal ID of the membership type.
    pub id: u64,
    /// Human-readable name.
    pub name: LocalizedString,
    /// Whether a membership of this type can be used by multiple persons.
    pub transferable: bool,
    /// Whether a membership of this type can be used for multiple parallel tickets.
    pub allow_parallel_usage: bool,
    /// Maximum number of times a membership of this type can be used (`None` =
    /// unlimited).
    pub max_usages: Option<u64>,
}

/// Body of [`MembershipTypes::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateMembershipType {
    /// Human-readable name.
    #[builder(into)]
    pub name: LocalizedString,
    /// Whether a membership of this type can be used by multiple persons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transferable: Option<bool>,
    /// Whether a membership of this type can be used for multiple parallel tickets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_parallel_usage: Option<bool>,
    /// Maximum number of usages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_usages: Option<u64>,
}

/// Body of [`MembershipTypes::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateMembershipType {
    /// Human-readable name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<LocalizedString>,
    /// Whether a membership of this type can be used by multiple persons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transferable: Option<bool>,
    /// Whether a membership of this type can be used for multiple parallel tickets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_parallel_usage: Option<bool>,
    /// Maximum number of usages (`Some(None)` = unlimited).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub max_usages: Option<Option<u64>>,
}

/// Endpoints for the membership types of an organizer. Created with
/// [`OrganizerScope::membership_types`].
#[derive(Debug, Clone)]
pub struct MembershipTypes {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Membership types of this organizer.
    #[must_use]
    pub fn membership_types(&self) -> MembershipTypes {
        MembershipTypes {
            client: self.client(),
            path: self.path().join("membershiptypes"),
        }
    }
}

impl MembershipTypes {
    /// Lists membership types.
    pub fn list(&self, params: &ListParams) -> Request<Page<MembershipType>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one membership type.
    pub fn get(&self, id: u64) -> Request<MembershipType> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a membership type.
    pub fn create(&self, body: &CreateMembershipType) -> Request<MembershipType> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a membership type.
    pub fn update(&self, id: u64, body: &UpdateMembershipType) -> Request<MembershipType> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a membership type. Types that have been used cannot be deleted.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
