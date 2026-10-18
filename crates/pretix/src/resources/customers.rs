//! Customer accounts.
//!
//! <https://docs.pretix.eu/dev/api/resources/customers.html>

use std::collections::BTreeMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::{Client, OrganizerScope, Page, Request};

/// A customer account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct Customer {
    /// Internal ID of the customer.
    pub identifier: String,
    /// External ID of the customer. Read-only for customers created through SSO.
    pub external_identifier: Option<String>,
    /// Email address.
    pub email: Option<String>,
    /// Phone number.
    #[serde(default)]
    pub phone: Option<String>,
    /// Name of the customer.
    pub name: Option<String>,
    /// Decomposition of the name (e.g. given name, family name).
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub name_parts: BTreeMap<String, String>,
    /// Whether this account is active.
    pub is_active: bool,
    /// Whether the email address has been verified.
    pub is_verified: bool,
    /// Time of the last login.
    pub last_login: Option<Timestamp>,
    /// Time of registration.
    pub date_joined: Timestamp,
    /// Preferred language.
    pub locale: String,
    /// Time of the last modification.
    pub last_modified: Timestamp,
    /// Internal notes and comments.
    pub notes: Option<String>,
}

/// Query parameters for [`Customers::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListCustomers {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Only customers with this email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub email: Option<String>,
}

/// Body of [`Customers::create`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateCustomer {
    /// External ID of the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub external_identifier: Option<String>,
    /// Email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub email: Option<String>,
    /// Phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub phone: Option<String>,
    /// Decomposition of the name (e.g. given name, family name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_parts: Option<BTreeMap<String, String>>,
    /// Whether this account is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    /// Whether the email address has been verified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_verified: Option<bool>,
    /// Preferred language.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub locale: Option<String>,
    /// Internal notes and comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub notes: Option<String>,
    /// Initial password.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub password: Option<String>,
    /// Send an account activation email with a password reset link (default `false`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_email: Option<bool>,
}

/// Body of [`Customers::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateCustomer {
    /// External ID of the customer (not for customers created through SSO).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub external_identifier: Option<Option<String>>,
    /// Email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub email: Option<String>,
    /// Phone number.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub phone: Option<Option<String>>,
    /// Decomposition of the name (e.g. given name, family name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_parts: Option<BTreeMap<String, String>>,
    /// Whether this account is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    /// Whether the email address has been verified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_verified: Option<bool>,
    /// Preferred language.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub locale: Option<String>,
    /// Internal notes and comments.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub notes: Option<Option<String>>,
}

/// Endpoints for the customer accounts of an organizer. Created with
/// [`OrganizerScope::customers`].
#[derive(Debug, Clone)]
pub struct Customers {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Customer accounts of this organizer.
    #[must_use]
    pub fn customers(&self) -> Customers {
        Customers {
            client: self.client(),
            path: self.path().join("customers"),
        }
    }
}

impl Customers {
    /// Lists customers.
    pub fn list(&self, params: &ListCustomers) -> Request<Page<Customer>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one customer.
    pub fn get(&self, identifier: &str) -> Request<Customer> {
        self.client.get(&self.path.join(identifier)).json()
    }

    /// Creates a customer.
    pub fn create(&self, body: &CreateCustomer) -> Request<Customer> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a customer.
    pub fn update(&self, identifier: &str, body: &UpdateCustomer) -> Request<Customer> {
        self.client
            .patch(&self.path.join(identifier))
            .json_body(body)
            .json()
    }

    /// Anonymizes a customer: deletes personal data and disconnects existing orders.
    pub fn anonymize(&self, identifier: &str) -> Request<Customer> {
        self.client
            .post(&self.path.join(identifier).join("anonymize"))
            .json()
    }
}
