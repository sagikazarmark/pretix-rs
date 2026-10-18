//! Exhibitors at trade shows and conferences, their scanned leads and vouchers
//! (plugin `pretix-exhibitors`).
//!
//! Only the REST API is covered. The App API used by the pretixLEAD app
//! (`/exhibitors/api/v1/`, authenticated with an exhibitor access code) is not.
//!
//! <https://docs.pretix.eu/dev/api/resources/exhibitors.html>

use std::collections::BTreeMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::resources::vouchers::Voucher;
use crate::{Client, EventScope, ListParams, Page, Request};

/// An exhibitor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Exhibitor {
    /// Internal exhibitor ID.
    pub id: u64,
    /// Exhibitor name.
    pub name: String,
    /// ID in your exhibition system, customer ID, etc.
    pub internal_id: Option<String>,
    /// Contact person.
    pub contact_name: Option<String>,
    /// Parts of the contact name (e.g. given name, family name), including `_scheme`.
    #[serde(default)]
    pub contact_name_parts: BTreeMap<String, String>,
    /// Contact person's email address.
    pub contact_email: Option<String>,
    /// Copy email addresses, separated by comma.
    pub contact_cc_email: Option<String>,
    /// Booth number.
    pub booth: Option<String>,
    /// Locale for communication with the exhibitor.
    pub locale: String,
    /// Access code for the exhibitor to access their data or use the lead scanning app.
    pub access_code: String,
    /// Access code for the lead scanning app only (no data access).
    pub lead_scanning_access_code: String,
    /// Whether the lead scanning app is enabled.
    pub allow_lead_scanning: bool,
    /// Whether access to data gathered by the lead scanning app is enabled.
    pub allow_lead_access: bool,
    /// Whether access to data gathered by exhibitor vouchers is enabled.
    pub allow_voucher_access: bool,
    /// Whether leads are handled as one lead per attendee per scanning device instead of
    /// per exhibitor.
    pub lead_scanning_scope_by_device: bool,
    /// Internal comment, not shown to the exhibitor.
    pub comment: String,
    /// Internal tags to categorize exhibitors, not shown to the exhibitor.
    #[serde(default)]
    pub exhibitor_tags: Vec<String>,
}

/// A lead scanned by an exhibitor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Lead {
    /// Order code of the scanned attendee's order.
    pub attendee_order: String,
    /// `positionid` of the attendee within the order.
    pub attendee_positionid: u64,
    /// A rating of 0 to 5 stars.
    pub rating: Option<u8>,
    /// A note taken by the exhibitor after scanning.
    pub notes: String,
    /// Additional tags selected by the exhibitor.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Time of the first upload of this lead.
    pub first_upload: Timestamp,
    /// Attendee data that may be shown to the exhibitor.
    #[serde(default)]
    pub data: Vec<LeadField>,
    /// User-defined name of the scanning device.
    #[serde(default)]
    pub device_name: Option<String>,
    /// UUID of the scanning device.
    #[serde(default)]
    pub device_uuid: Option<String>,
}

/// An attendee data field of a [`Lead`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LeadField {
    /// Internal field ID, e.g. `attendee_name`.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// The value.
    pub value: Option<String>,
    /// Parts of the value (e.g. name parts). Usually empty.
    #[serde(default)]
    pub details: Vec<LeadFieldDetail>,
}

/// A part of a [`LeadField`] value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LeadFieldDetail {
    /// Human-readable label.
    pub label: String,
    /// The value.
    pub value: Option<String>,
}

/// A voucher connected to an exhibitor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExhibitorVoucher {
    /// Comment shown to the exhibitor. Can only be changed with
    /// [`ExhibitorVouchers::attach`].
    #[serde(default)]
    pub exhibitor_comment: Option<String>,
    /// The voucher.
    #[serde(flatten)]
    pub voucher: Voucher,
}

/// Body of [`Exhibitors::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateExhibitor {
    /// Exhibitor name.
    #[builder(into)]
    pub name: String,
    /// ID in your exhibition system, customer ID, etc. (at most 255 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub internal_id: Option<String>,
    /// Contact person.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub contact_name: Option<String>,
    /// Parts of the contact name, including `_scheme`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_name_parts: Option<BTreeMap<String, String>>,
    /// Contact person's email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub contact_email: Option<String>,
    /// Copy email addresses, separated by comma.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub contact_cc_email: Option<String>,
    /// Booth number (at most 100 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub booth: Option<String>,
    /// Locale for communication with the exhibitor.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub locale: Option<String>,
    /// Whether the lead scanning app is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_lead_scanning: Option<bool>,
    /// Whether access to data gathered by the lead scanning app is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_lead_access: Option<bool>,
    /// Whether access to data gathered by exhibitor vouchers is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_voucher_access: Option<bool>,
    /// Whether leads are handled as one lead per attendee per scanning device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_scanning_scope_by_device: Option<bool>,
    /// Internal comment, not shown to the exhibitor.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub comment: Option<String>,
    /// Internal tags (must already exist).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exhibitor_tags: Option<Vec<String>>,
}

/// Body of [`Exhibitors::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateExhibitor {
    /// Exhibitor name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<String>,
    /// ID in your exhibition system, customer ID, etc. (at most 255 characters).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub internal_id: Option<Option<String>>,
    /// Contact person.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub contact_name: Option<Option<String>>,
    /// Parts of the contact name, including `_scheme`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_name_parts: Option<BTreeMap<String, String>>,
    /// Contact person's email address.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub contact_email: Option<Option<String>>,
    /// Copy email addresses, separated by comma.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub contact_cc_email: Option<Option<String>>,
    /// Booth number (at most 100 characters).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub booth: Option<Option<String>>,
    /// Locale for communication with the exhibitor.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub locale: Option<String>,
    /// Whether the lead scanning app is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_lead_scanning: Option<bool>,
    /// Whether access to data gathered by the lead scanning app is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_lead_access: Option<bool>,
    /// Whether access to data gathered by exhibitor vouchers is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_voucher_access: Option<bool>,
    /// Whether leads are handled as one lead per attendee per scanning device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_scanning_scope_by_device: Option<bool>,
    /// Internal comment, not shown to the exhibitor.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub comment: Option<String>,
    /// Internal tags (must already exist).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exhibitor_tags: Option<Vec<String>>,
}

/// Body of [`ExhibitorVouchers::attach`] and [`ExhibitorVouchers::bulk_attach`]: an existing voucher,
/// identified by either `id` or `code`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct AttachVoucher {
    /// Internal voucher ID. Set either this or `code`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    /// Voucher code. Set either this or `id`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub code: Option<String>,
    /// Comment shown to the exhibitor.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub exhibitor_comment: Option<String>,
}

/// Endpoints for the exhibitors of an event. Created with [`EventScope::exhibitors`].
#[derive(Debug, Clone)]
pub struct Exhibitors {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Exhibitors of this event.
    #[must_use]
    pub fn exhibitors(&self) -> Exhibitors {
        Exhibitors {
            client: self.client(),
            path: self.path().join("exhibitors"),
        }
    }
}

impl Exhibitors {
    /// Lists exhibitors.
    pub fn list(&self, params: &ListParams) -> Request<Page<Exhibitor>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one exhibitor.
    pub fn get(&self, id: u64) -> Request<Exhibitor> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates an exhibitor.
    pub fn create(&self, body: &CreateExhibitor) -> Request<Exhibitor> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates an exhibitor.
    pub fn update(&self, id: u64, body: &UpdateExhibitor) -> Request<Exhibitor> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes an exhibitor, including all lead scan data. Connected vouchers are
    /// detached, not deleted.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }

    /// Emails the exhibitor their access code.
    pub fn send_access_code(&self, id: u64) -> Request<()> {
        self.client
            .post(&self.path.join(id).join("send_access_code"))
            .empty()
    }

    /// Lists the leads scanned by an exhibitor.
    pub fn leads(&self, id: u64, params: &ListParams) -> Request<Page<Lead>> {
        self.client
            .get(&self.path.join(id).join("leads"))
            .query(params)
            .page()
    }

    /// Vouchers connected to the exhibitor `exhibitor_id`.
    #[must_use]
    pub fn vouchers(&self, exhibitor_id: u64) -> ExhibitorVouchers {
        ExhibitorVouchers {
            client: self.client.clone(),
            path: self.path.join(exhibitor_id).join("vouchers"),
        }
    }
}

/// Endpoints for the vouchers connected to an exhibitor. Created with
/// [`Exhibitors::vouchers`].
#[derive(Debug, Clone)]
pub struct ExhibitorVouchers {
    client: Client,
    path: Path,
}

impl ExhibitorVouchers {
    /// Lists the vouchers connected to the exhibitor.
    pub fn list(&self, params: &ListParams) -> Request<Page<ExhibitorVoucher>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Attaches an existing voucher to the exhibitor, or updates its
    /// `exhibitor_comment`.
    pub fn attach(&self, body: &AttachVoucher) -> Request<()> {
        self.client
            .post(&self.path.join("attach"))
            .json_body(body)
            .empty()
    }

    /// Attaches many existing vouchers to the exhibitor. All entries must identify the
    /// voucher by the same field (`id` or `code`).
    pub fn bulk_attach(&self, body: &[AttachVoucher]) -> Request<()> {
        self.client
            .post(&self.path.join("bulk_attach"))
            .json_body(body)
            .empty()
    }
}
