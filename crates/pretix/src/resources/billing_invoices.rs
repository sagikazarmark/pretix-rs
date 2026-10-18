//! Invoices for pretix Hosted itself (pretix Hosted only).
//!
//! <https://docs.pretix.eu/dev/api/resources/billing_invoices.html>

use http::StatusCode;
use jiff::civil::Date;
use serde::{Deserialize, Serialize};

use crate::job::JobCodes;
use crate::path::Path;
use crate::query::Ordering;
use crate::types::string_enum;
use crate::{Client, Download, JobStatus, OrganizerScope, Page, Request};

/// An invoice issued by pretix Hosted to the organizer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct BillingInvoice {
    /// Invoice number.
    pub invoice_number: String,
    /// Invoice date.
    pub date_issued: Date,
}

string_enum! {
    /// Fields billing invoices can be sorted by.
    pub enum BillingInvoiceOrdering {
        /// By invoice date (default).
        DateIssued = "date_issued",
    }
}

/// Query parameters for [`BillingInvoices::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListBillingInvoices {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<BillingInvoiceOrdering>>,
}

/// Endpoints for pretix Hosted invoices of an organizer. Created with
/// [`OrganizerScope::billing_invoices`].
#[derive(Debug, Clone)]
pub struct BillingInvoices {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Invoices pretix Hosted issued to this organizer (since November 2017).
    #[must_use]
    pub fn billing_invoices(&self) -> BillingInvoices {
        BillingInvoices {
            client: self.client(),
            path: self.path().join("billing_invoices"),
        }
    }
}

impl BillingInvoices {
    /// Lists invoices.
    pub fn list(&self, params: &ListBillingInvoices) -> Request<Page<BillingInvoice>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one invoice.
    pub fn get(&self, invoice_number: &str) -> Request<BillingInvoice> {
        self.client.get(&self.path.join(invoice_number)).json()
    }

    /// Downloads the PDF of an invoice.
    ///
    /// New invoices are reviewed before they can be downloaded, which usually takes a
    /// few hours; until then pretix answers `423 Locked`, which is reported as
    /// [`JobStatus::Pending`].
    pub fn download(&self, invoice_number: &str) -> Request<JobStatus<Download>> {
        self.client
            .get(&self.path.join(invoice_number).join("download"))
            .job_with(
                JobCodes {
                    pending: &[StatusCode::CONFLICT, StatusCode::LOCKED],
                    failed: &[],
                },
                Download::from_response,
            )
    }
}
