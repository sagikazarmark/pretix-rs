//! Imported ticket secrets used instead of pretix-generated ones (plugin
//! `pretix-secrets-import`).
//!
//! <https://docs.pretix.eu/dev/api/resources/imported_secrets.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::{Client, EventScope, ListParams, Page, Request};

/// An imported ticket secret.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ImportedSecret {
    /// Internal ID of the secret.
    pub id: u64,
    /// The secret (QR code content).
    pub secret: String,
    /// Whether the secret was already used for a ticket. Used secrets cannot be deleted
    /// and are never used twice.
    pub used: bool,
    /// Product ID the secret is restricted to.
    pub item: Option<u64>,
    /// Product variation ID the secret is restricted to.
    pub variation: Option<u64>,
    /// Event series date ID the secret is restricted to.
    pub subevent: Option<u64>,
}

/// Body of [`ImportedSecrets::create`] and [`ImportedSecrets::bulk_create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateImportedSecret {
    /// The secret (QR code content).
    #[builder(into)]
    pub secret: String,
    /// Whether the secret was already used for a ticket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used: Option<bool>,
    /// Only use the secret for tickets of this product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<u64>,
    /// Only use the secret for tickets of this product variation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// Only use the secret for tickets of this event series date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
}

/// Body of [`ImportedSecrets::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateImportedSecret {
    /// The secret (QR code content).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub secret: Option<String>,
    /// Whether the secret was already used for a ticket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used: Option<bool>,
    /// Only use the secret for tickets of this product.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub item: Option<Option<u64>>,
    /// Only use the secret for tickets of this product variation.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub variation: Option<Option<u64>>,
    /// Only use the secret for tickets of this event series date.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub subevent: Option<Option<u64>>,
}

/// Endpoints for the imported secrets of an event. Created with
/// [`EventScope::imported_secrets`].
#[derive(Debug, Clone)]
pub struct ImportedSecrets {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Imported ticket secrets of this event.
    #[must_use]
    pub fn imported_secrets(&self) -> ImportedSecrets {
        ImportedSecrets {
            client: self.client(),
            path: self.path().join("imported_secrets"),
        }
    }
}

impl ImportedSecrets {
    /// Lists imported secrets.
    pub fn list(&self, params: &ListParams) -> Request<Page<ImportedSecret>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one imported secret.
    pub fn get(&self, id: u64) -> Request<ImportedSecret> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Imports a secret.
    pub fn create(&self, body: &CreateImportedSecret) -> Request<ImportedSecret> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Imports up to 500 secrets at once. Either all or none are created.
    pub fn bulk_create(&self, body: &[CreateImportedSecret]) -> Request<Vec<ImportedSecret>> {
        self.client
            .post(&self.path.join("bulk_create"))
            .json_body(body)
            .json()
    }

    /// Updates an imported secret.
    pub fn update(&self, id: u64, body: &UpdateImportedSecret) -> Request<ImportedSecret> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes an imported secret. Used secrets cannot be deleted.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
