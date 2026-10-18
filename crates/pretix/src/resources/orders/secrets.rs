//! Revoked and blocked ticket secrets.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Ordering;
use crate::types::string_enum;
use crate::{Client, EventScope, Page, Request};

/// A revoked ticket secret (needed for validation with some secret generators).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct RevokedSecret {
    /// Internal ID.
    pub id: u64,
    /// The revoked secret.
    pub secret: String,
    /// Time of revocation.
    pub created: Timestamp,
}

/// A currently or formerly blocked ticket secret.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct BlockedSecret {
    /// Internal ID.
    pub id: u64,
    /// The secret.
    pub secret: String,
    /// Whether the secret is currently blocked.
    pub blocked: bool,
    /// Time of the last update.
    pub updated: Timestamp,
}

string_enum! {
    /// Fields revoked secrets can be sorted by.
    pub enum RevokedSecretOrdering {
        /// By secret.
        Secret = "secret",
        /// By revocation time (default: descending).
        Created = "created",
    }
}

/// Query parameters for [`RevokedSecrets::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListRevokedSecrets {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<RevokedSecretOrdering>>,
    /// Only revocations created since this time. Use the
    /// [`page_generated`](crate::ResponseMeta::page_generated) time of a previous list
    /// response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_since: Option<Timestamp>,
}

/// Query parameters for [`BlockedSecrets::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListBlockedSecrets {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Only records updated since this time. Use the
    /// [`page_generated`](crate::ResponseMeta::page_generated) time of a previous list
    /// response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_since: Option<Timestamp>,
    /// Only blocked (`true`) or unblocked (`false`) records.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked: Option<bool>,
}

/// Endpoints for revoked ticket secrets. Created with [`EventScope::revoked_secrets`].
#[derive(Debug, Clone)]
pub struct RevokedSecrets {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Revoked ticket secrets of this event.
    #[must_use]
    pub fn revoked_secrets(&self) -> RevokedSecrets {
        RevokedSecrets {
            client: self.client(),
            path: self.path().join("revokedsecrets"),
        }
    }

    /// Blocked (and formerly blocked) ticket secrets of this event.
    #[must_use]
    pub fn blocked_secrets(&self) -> BlockedSecrets {
        BlockedSecrets {
            client: self.client(),
            path: self.path().join("blockedsecrets"),
        }
    }
}

impl RevokedSecrets {
    /// Lists revoked secrets.
    pub fn list(&self, params: &ListRevokedSecrets) -> Request<Page<RevokedSecret>> {
        self.client.get(&self.path).query(params).page()
    }
}

/// Endpoints for blocked ticket secrets. Created with [`EventScope::blocked_secrets`].
#[derive(Debug, Clone)]
pub struct BlockedSecrets {
    client: Client,
    path: Path,
}

impl BlockedSecrets {
    /// Lists blocked and formerly blocked secrets, most recently updated first. Check
    /// `blocked` for the current state.
    pub fn list(&self, params: &ListBlockedSecrets) -> Request<Page<BlockedSecret>> {
        self.client.get(&self.path).query(params).page()
    }
}
