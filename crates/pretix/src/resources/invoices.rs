//! Invoices.
//!
//! <https://docs.pretix.eu/dev/api/resources/invoices.html>

use jiff::Timestamp;
use jiff::civil::Date;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Ordering;
use crate::resources::orders::FeeType;
use crate::types::{Decimal, string_enum};
use crate::{Client, Download, EventScope, JobStatus, OrganizerScope, Page, Request};

/// An invoice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Invoice {
    /// Invoice number (with prefix).
    pub number: String,
    /// The slug of the parent event.
    pub event: String,
    /// Code of the order this invoice belongs to.
    pub order: String,
    /// Whether this invoice cancels a different invoice.
    pub is_cancellation: bool,
    /// Sender address: name.
    #[serde(default)]
    pub invoice_from_name: Option<String>,
    /// Sender address: address lines.
    #[serde(default)]
    pub invoice_from: Option<String>,
    /// Sender address: ZIP code.
    #[serde(default)]
    pub invoice_from_zipcode: Option<String>,
    /// Sender address: city.
    #[serde(default)]
    pub invoice_from_city: Option<String>,
    /// Sender address: state (only used in some countries).
    #[serde(default)]
    pub invoice_from_state: Option<String>,
    /// Sender address: country code.
    #[serde(default)]
    pub invoice_from_country: Option<String>,
    /// Sender address: local tax ID.
    #[serde(default)]
    pub invoice_from_tax_id: Option<String>,
    /// Sender address: EU VAT ID.
    #[serde(default)]
    pub invoice_from_vat_id: Option<String>,
    /// Full recipient address.
    pub invoice_to: String,
    /// Recipient: business or individual (`None` for invoices created before pretix
    /// 2025.6).
    #[serde(default)]
    pub invoice_to_is_business: Option<bool>,
    /// Recipient: company name.
    #[serde(default)]
    pub invoice_to_company: Option<String>,
    /// Recipient: person name.
    #[serde(default)]
    pub invoice_to_name: Option<String>,
    /// Recipient: address lines.
    #[serde(default)]
    pub invoice_to_street: Option<String>,
    /// Recipient: ZIP code.
    #[serde(default)]
    pub invoice_to_zipcode: Option<String>,
    /// Recipient: city.
    #[serde(default)]
    pub invoice_to_city: Option<String>,
    /// Recipient: state (only used in some countries).
    #[serde(default)]
    pub invoice_to_state: Option<String>,
    /// Recipient: country code.
    #[serde(default)]
    pub invoice_to_country: Option<String>,
    /// Recipient: EU VAT ID.
    #[serde(default)]
    pub invoice_to_vat_id: Option<String>,
    /// Invoice beneficiary.
    #[serde(default)]
    pub invoice_to_beneficiary: Option<String>,
    /// Additional transmission info (depends on the transmission type).
    #[serde(default)]
    pub invoice_to_transmission_info: Option<serde_json::Map<String, serde_json::Value>>,
    /// Custom invoice address field.
    #[serde(default)]
    pub custom_field: Option<String>,
    /// Invoice date.
    pub date: Date,
    /// Number of the invoice this invoice refers to (e.g. the canceled invoice).
    pub refers: Option<String>,
    /// Invoice locale.
    pub locale: String,
    /// Text printed above the product list.
    #[serde(default)]
    pub introductory_text: String,
    /// Text printed below the product list.
    #[serde(default)]
    pub additional_text: String,
    /// Payment information printed below the product list.
    #[serde(default)]
    pub payment_provider_text: String,
    /// Short text visibly indicating the payment status.
    #[serde(default)]
    pub payment_provider_stamp: Option<String>,
    /// Text printed in the page footer.
    #[serde(default)]
    pub footer_text: String,
    /// The invoice lines.
    #[serde(default)]
    pub lines: Vec<InvoiceLine>,
    /// Currency code of a second currency to show totals in.
    #[serde(default)]
    pub foreign_currency_display: Option<String>,
    /// Exchange rate to the event currency at invoicing time, if known.
    #[serde(default)]
    pub foreign_currency_rate: Option<Decimal>,
    /// Date the exchange rate was obtained.
    #[serde(default)]
    pub foreign_currency_rate_date: Option<Date>,
    /// Customer's reference printed on the invoice.
    #[serde(default)]
    pub internal_reference: String,
    /// Requested transmission channel.
    #[serde(default)]
    pub transmission_type: Option<TransmissionType>,
    /// Selected transmission provider (depends on installed plugins).
    #[serde(default)]
    pub transmission_provider: Option<String>,
    /// Transmission status.
    #[serde(default)]
    pub transmission_status: Option<TransmissionStatus>,
    /// Time of the last change of the transmission status.
    #[serde(default)]
    pub transmission_date: Option<Timestamp>,
}

/// A line of an [`Invoice`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InvoiceLine {
    /// Number of the line within the invoice.
    pub position: u32,
    /// Text of the line (e.g. product name).
    pub description: String,
    /// ID of the product this line was created from (may have changed since).
    #[serde(default)]
    pub item: Option<u64>,
    /// ID of the product variation this line was created from.
    #[serde(default)]
    pub variation: Option<u64>,
    /// ID of the event series date this line was created from.
    #[serde(default)]
    pub subevent: Option<u64>,
    /// Fee type, for lines created from a fee.
    #[serde(default)]
    pub fee_type: Option<FeeType>,
    /// Additional fee type (e.g. the payment provider), for lines created from a fee.
    #[serde(default)]
    pub fee_internal_type: Option<String>,
    /// Start of the service or delivery period.
    #[serde(default)]
    pub period_start: Option<Timestamp>,
    /// End of the service or delivery period.
    #[serde(default)]
    pub period_end: Option<Timestamp>,
    /// **Deprecated** alias of `period_start`.
    #[serde(default)]
    pub event_date_from: Option<Timestamp>,
    /// **Deprecated** alias of `period_end`.
    #[serde(default)]
    pub event_date_to: Option<Timestamp>,
    /// Location of the (sub-)event at invoice creation.
    #[serde(default)]
    pub event_location: Option<String>,
    /// Attendee name at invoice creation.
    #[serde(default)]
    pub attendee_name: Option<String>,
    /// Price including taxes.
    pub gross_value: Decimal,
    /// Tax amount included.
    pub tax_value: Decimal,
    /// Name of the tax rate (e.g. `VAT`).
    #[serde(default)]
    pub tax_name: String,
    /// Codified reason for the tax rate.
    #[serde(default)]
    pub tax_code: Option<String>,
    /// Tax rate.
    pub tax_rate: Decimal,
}

string_enum! {
    /// How an invoice is transmitted to the recipient.
    pub enum TransmissionType {
        /// PDF by email (default).
        Email = "email",
        /// XML through the Peppol network (requires a plugin).
        Peppol = "peppol",
        /// XML through the Italian Sistema di Interscambio (requires a plugin).
        ItSdi = "it_sdi",
    }
}

string_enum! {
    /// The transmission status of an invoice.
    pub enum TransmissionStatus {
        /// Unknown (invoices created before pretix 2025.6).
        Unknown = "unknown",
        /// Not yet transmitted.
        Pending = "pending",
        /// Currently being transmitted.
        Inflight = "inflight",
        /// Transmission failed.
        Failed = "failed",
        /// Transmitted.
        Completed = "completed",
    }
}

string_enum! {
    /// Fields invoices can be sorted by.
    pub enum InvoiceOrdering {
        /// By date.
        Date = "date",
        /// By invoice number (default).
        Number = "nr",
    }
}

/// Query parameters for [`Invoices::list`] and [`OrganizerInvoices::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct ListInvoices {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<InvoiceOrdering>>,
    /// Only invoices with this `is_cancellation` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_cancellation: Option<bool>,
    /// Only invoices of one of these order codes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub order: Vec<String>,
    /// Only invoices with one of these numbers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub number: Vec<String>,
    /// Only invoices referring to this invoice number.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub refers: Option<String>,
    /// Only invoices with this locale.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub locale: Option<String>,
}

/// Endpoints for the invoices of an event. Created with [`EventScope::invoices`].
#[derive(Debug, Clone)]
pub struct Invoices {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Invoices of this event.
    #[must_use]
    pub fn invoices(&self) -> Invoices {
        Invoices {
            client: self.client(),
            path: self.path().join("invoices"),
        }
    }
}

impl Invoices {
    /// Lists invoices.
    pub fn list(&self, params: &ListInvoices) -> Request<Page<Invoice>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one invoice by its number.
    pub fn get(&self, number: &str) -> Request<Invoice> {
        self.client.get(&self.path.join(number)).json()
    }

    /// Downloads the invoice PDF.
    ///
    /// Returns [`JobStatus::Pending`] while the file is being generated; use
    /// [`Request::wait`] to poll until it is ready.
    pub fn download(&self, number: &str) -> Request<JobStatus<Download>> {
        self.client
            .get(&self.path.join(number).join("download"))
            .download_job()
    }

    /// Cancels the invoice and creates a new one.
    pub fn reissue(&self, number: &str) -> Request<()> {
        self.client
            .post(&self.path.join(number).join("reissue"))
            .empty()
    }

    /// Re-generates the invoice from the order data.
    pub fn regenerate(&self, number: &str) -> Request<()> {
        self.client
            .post(&self.path.join(number).join("regenerate"))
            .empty()
    }

    /// Transmits the invoice to the recipient if it is `pending`.
    pub fn transmit(&self, number: &str) -> Request<()> {
        self.client
            .post(&self.path.join(number).join("transmit"))
            .empty()
    }

    /// Transmits the invoice again, even if transmission was attempted before.
    pub fn retransmit(&self, number: &str) -> Request<()> {
        self.client
            .post(&self.path.join(number).join("retransmit"))
            .empty()
    }
}

/// Organizer-wide list of invoices across all events. Created with
/// [`OrganizerScope::invoices`].
#[derive(Debug, Clone)]
pub struct OrganizerInvoices {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Invoices of all events of this organizer.
    #[must_use]
    pub fn invoices(&self) -> OrganizerInvoices {
        OrganizerInvoices {
            client: self.client(),
            path: self.path().join("invoices"),
        }
    }
}

impl OrganizerInvoices {
    /// Lists invoices of all events (with sufficient permissions).
    pub fn list(&self, params: &ListInvoices) -> Request<Page<Invoice>> {
        self.client.get(&self.path).query(params).page()
    }
}
