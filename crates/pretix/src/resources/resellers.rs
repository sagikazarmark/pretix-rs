//! Reseller accounts of value-added reseller partners (pretix Hosted only).
//!
//! This API requires an OAuth access token of a user with access to the reseller
//! account; team API tokens are not accepted.
//!
//! <https://docs.pretix.eu/dev/api/resources/billing_var.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::{Client, ListParams, Page, Request};

/// A reseller account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Reseller {
    /// Reseller ID.
    pub id: u64,
    /// Internal name of the reseller account.
    pub name: String,
    /// Public name of the reseller account.
    pub public_name: String,
    /// Public URL of the reseller's company.
    pub public_url: String,
    /// Support email address.
    pub support_email: String,
    /// Support phone number.
    pub support_phone: String,
    /// Language code used to communicate with the reseller.
    pub communication_language: String,
}

/// An organizer account created with [`Resellers::create_organizer`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ResellerOrganizer {
    /// Internal organizer ID.
    pub id: u64,
    /// Name of the organizer.
    pub name: String,
    /// Short form of the name, used in URLs.
    pub slug: String,
}

/// Body of [`Resellers::create_organizer`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateOrganizer {
    /// Name of the organizer.
    #[builder(into)]
    pub name: String,
    /// Short form of the name, used in URLs.
    #[builder(into)]
    pub slug: String,
}

/// Endpoints for reseller accounts. Created with [`Client::resellers`].
#[derive(Debug, Clone)]
pub struct Resellers {
    client: Client,
    path: Path,
}

impl Client {
    /// Reseller accounts the user has access to.
    #[must_use]
    pub fn resellers(&self) -> Resellers {
        Resellers {
            client: self.clone(),
            path: Path::root().join("var"),
        }
    }
}

impl Resellers {
    /// Lists reseller accounts.
    pub fn list(&self, params: &ListParams) -> Request<Page<Reseller>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one reseller account.
    pub fn get(&self, id: u64) -> Request<Reseller> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates an organizer account associated with a reseller account.
    pub fn create_organizer(&self, id: u64, body: &CreateOrganizer) -> Request<ResellerOrganizer> {
        self.client
            .post(&self.path.join(id).join("create_organizer"))
            .json_body(body)
            .json()
    }
}
