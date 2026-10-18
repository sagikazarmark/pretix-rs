//! Bank transfer import jobs of the bundled bank transfer plugin, used e.g. by
//! `pretix-banktool` to upload bank transactions.
//!
//! <https://docs.pretix.eu/dev/api/resources/banktransfer.html>

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::types::{Decimal, string_enum};
use crate::{Client, OrganizerScope, Page, Request};

/// A bank import job: a batch of uploaded bank transactions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct BankImportJob {
    /// Internal job ID.
    pub id: u64,
    /// Slug of the event the job was uploaded for.
    pub event: Option<String>,
    /// Job creation time.
    pub created: Timestamp,
    /// Job state.
    pub state: BankImportJobState,
    /// Transactions of the job. Only filled once the job has started processing.
    #[serde(default)]
    pub transactions: Vec<BankTransaction>,
    /// Currency of the transactions (not in the pretix docs).
    #[serde(default)]
    pub currency: Option<String>,
}

/// A bank transaction of a [`BankImportJob`].
///
/// `payer` and `reference` are emptied once the transaction is matched to an order or
/// discarded; `checksum` persists for deduplication.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct BankTransaction {
    /// Transaction state.
    pub state: BankTransactionState,
    /// Error message, if any.
    pub message: String,
    /// Checksum computed from payer, reference, amount and date.
    pub checksum: String,
    /// Payment source.
    pub payer: String,
    /// Unique ID of the payment from an external source.
    pub external_id: Option<String>,
    /// Payment reference.
    pub reference: String,
    /// Payment amount.
    pub amount: Decimal,
    /// Payment IBAN.
    pub iban: String,
    /// Payment BIC.
    pub bic: String,
    /// Payment date, in the format it was uploaded in.
    pub date: String,
    /// Code of the associated order.
    pub order: Option<String>,
    /// Internal comment.
    pub comment: String,
    /// Currency of the payment (not in the pretix docs).
    #[serde(default)]
    pub currency: Option<String>,
}

string_enum! {
    /// The state of a [`BankImportJob`].
    pub enum BankImportJobState {
        /// Waiting to be processed.
        Pending = "pending",
        /// Being processed.
        Running = "running",
        /// Processing failed.
        Error = "error",
        /// Processing completed.
        Completed = "completed",
    }
}

string_enum! {
    /// The state of a [`BankTransaction`].
    pub enum BankTransactionState {
        /// Imported, not yet processed.
        Imported = "imported",
        /// No matching order found.
        NoMatch = "nomatch",
        /// Invalid, e.g. the amount does not match.
        Invalid = "invalid",
        /// An error occurred.
        Error = "error",
        /// Matched to an order.
        Valid = "valid",
        /// Discarded manually.
        Discarded = "discarded",
        /// The order was already paid.
        Already = "already",
    }
}

/// Query parameters for [`BankImportJobs::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListBankImportJobs {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Only jobs for the event with this slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub event: Option<String>,
    /// Only jobs in this state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<BankImportJobState>,
}

/// Body of [`BankImportJobs::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateBankImportJob {
    /// Slug of the event the transactions belong to. Without an event, transactions
    /// are matched against all events of the organizer.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub event: Option<String>,
    /// The transactions to import.
    pub transactions: Vec<CreateBankTransaction>,
    /// Currency of the transactions. Required without `event` if the organizer's events
    /// use different currencies (not in the pretix docs).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub currency: Option<String>,
}

/// A transaction in [`CreateBankImportJob`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateBankTransaction {
    /// Payment source.
    #[builder(into)]
    pub payer: String,
    /// Payment reference.
    #[builder(into)]
    pub reference: String,
    /// Payment amount.
    #[builder(into)]
    pub amount: Decimal,
    /// Payment date.
    #[builder(into)]
    pub date: String,
    /// Unique ID of the payment from an external source.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub external_id: Option<String>,
    /// Payment IBAN.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub iban: Option<String>,
    /// Payment BIC.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub bic: Option<String>,
    /// Internal comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub comment: Option<String>,
    /// Currency of the payment (not in the pretix docs).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub currency: Option<String>,
}

/// Endpoints for bank import jobs. Created with [`OrganizerScope::bank_import_jobs`].
#[derive(Debug, Clone)]
pub struct BankImportJobs {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Bank import jobs of this organizer.
    #[must_use]
    pub fn bank_import_jobs(&self) -> BankImportJobs {
        BankImportJobs {
            client: self.client(),
            path: self.path().join("bankimportjobs"),
        }
    }
}

impl BankImportJobs {
    /// Lists the bank import jobs the credentials have access to.
    pub fn list(&self, params: &ListBankImportJobs) -> Request<Page<BankImportJob>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one job.
    pub fn get(&self, id: u64) -> Request<BankImportJob> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Uploads transactions and starts processing them.
    ///
    /// Depending on the server, the job is processed immediately (state `completed` or
    /// `error`) or queued (state `pending`, no transactions yet; poll with
    /// [`BankImportJobs::get`]).
    pub fn create(&self, body: &CreateBankImportJob) -> Request<BankImportJob> {
        self.client.post(&self.path).json_body(body).json()
    }
}
