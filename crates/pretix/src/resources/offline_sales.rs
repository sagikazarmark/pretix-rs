//! Offline sales: batches of tickets intended for sale outside of pretix (plugin
//! `pretix-offlinesales`).
//!
//! <https://docs.pretix.eu/dev/api/resources/offlinesales.html>

use http::Method;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::path::Path;
use crate::request::decode;
use crate::resources::exporters::ExportJob;
use crate::{Client, EventScope, JobStatus, ListParams, Page, Request};

/// A batch of tickets for offline sale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct OfflineSalesBatch {
    /// Internal batch ID.
    pub id: u64,
    /// Time of creation.
    pub creation: Timestamp,
    /// Whether the orders are created in test mode.
    pub testmode: bool,
    /// Sales channel of the orders.
    pub sales_channel: String,
    /// ID of the ticket layout.
    pub layout: u64,
    /// ID of the event series date.
    pub subevent: Option<u64>,
    /// ID of the product.
    pub item: u64,
    /// ID of the product variation.
    pub variation: Option<u64>,
    /// Number of tickets in the batch.
    pub amount: u32,
    /// Internal comment.
    pub comment: String,
    /// Order codes. Only included in detail responses.
    #[serde(default)]
    pub orders: Vec<String>,
}

/// Response of [`OfflineSalesBatches::create`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct OfflineSalesBatchCreation {
    /// Where to poll for the created batch, see [`OfflineSalesBatches::check`].
    pub check: Url,
}

/// Body of [`OfflineSalesBatches::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateOfflineSalesBatch {
    /// ID of the product.
    pub item: u64,
    /// ID of the ticket layout.
    pub layout: u64,
    /// Number of tickets (at most 10,000).
    pub amount: u32,
    /// Whether to create the orders in test mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testmode: Option<bool>,
    /// Sales channel of the orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub sales_channel: Option<String>,
    /// ID of the event series date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subevent: Option<u64>,
    /// ID of the product variation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variation: Option<u64>,
    /// Internal comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub comment: Option<String>,
}

/// Endpoints for the offline sales batches of an event. Created with
/// [`EventScope::offline_sales_batches`].
#[derive(Debug, Clone)]
pub struct OfflineSalesBatches {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Offline sales batches of this event.
    #[must_use]
    pub fn offline_sales_batches(&self) -> OfflineSalesBatches {
        OfflineSalesBatches {
            client: self.client(),
            path: self.path().join("offlinesalesbatches"),
        }
    }
}

impl OfflineSalesBatches {
    /// Lists batches. `orders` is not included.
    pub fn list(&self, params: &ListParams) -> Request<Page<OfflineSalesBatch>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one batch.
    pub fn get(&self, id: u64) -> Request<OfflineSalesBatch> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Starts creating a batch in the background. Poll the result with
    /// [`OfflineSalesBatches::check`].
    pub fn create(&self, body: &CreateOfflineSalesBatch) -> Request<OfflineSalesBatchCreation> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Polls a batch creation job started with [`OfflineSalesBatches::create`] once.
    ///
    /// Use [`Request::wait`] to poll until the batch is created.
    pub fn check(&self, job: &OfflineSalesBatchCreation) -> Request<JobStatus<OfflineSalesBatch>> {
        self.client
            .request_absolute(Method::GET, &job.check)
            .job(decode::json)
    }

    /// Starts rendering the PDF of a batch in the background.
    ///
    /// Poll the result with [`ExportJob::download_job`].
    pub fn render(&self, id: u64) -> Request<ExportJob> {
        self.client.post(&self.path.join(id).join("render")).json()
    }
}
