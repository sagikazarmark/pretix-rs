//! Data shredders: deleting personal data of an event.
//!
//! Shredding is a multi-step process:
//!
//! 1. [`Shredders::export`] starts an export of the data to be deleted and returns a
//!    [`ShredderExport`].
//! 2. Optionally download the export ([`ShredderExport::download_job`]).
//! 3. [`Shredders::shred`] starts the deletion once the export is ready and returns a
//!    [`ShredJob`].
//! 4. [`Shredders::status`] reports whether the deletion succeeded.
//!
//! Unlike the web interface, the API does not force you to download tax-relevant data
//! before deleting it.
//!
//! <https://docs.pretix.eu/dev/api/resources/shredders.html>

use http::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::job::JobCodes;
use crate::path::Path;
use crate::request::decode;
use crate::{Client, Download, EventScope, JobStatus, ListParams, Page, Request};

/// A data shredder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Shredder {
    /// Identifier of the shredder, used with [`Shredders::export`].
    pub identifier: String,
    /// Human-readable name.
    pub verbose_name: String,
}

/// A started shredder export, returned by [`Shredders::export`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ShredderExport {
    /// Where to download the exported data. Responds with `409` while the export is
    /// running. Not available any more after shredding.
    pub download: Url,
    /// Where to start shredding, see [`Shredders::shred`].
    pub shred: Url,
}

impl ShredderExport {
    /// Polls the exported data once. Equivalent to
    /// `client.download_job(&self.download)`.
    pub fn download_job(&self, client: &Client) -> Request<JobStatus<Download>> {
        client.download_job(&self.download)
    }
}

/// A started shredding process, returned by [`Shredders::shred`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ShredJob {
    /// Where to check the result, see [`Shredders::status`].
    pub status: Url,
}

/// The outcome of a finished shredding process, see [`Shredders::status`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ShredOutcome {
    /// The data was deleted (`200`).
    Completed,
    /// pretix no longer knows about the process (`410`), e.g. because it was started
    /// more than an hour ago. Also returned after successful runs on installations
    /// without asynchronous task processing.
    Unknown,
}

/// Body of [`Shredders::export`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct ExportShredders {
    /// Identifiers of the shredders to run.
    pub shredders: Vec<String>,
}

/// Endpoints for the data shredders of an event. Created with
/// [`EventScope::shredders`].
#[derive(Debug, Clone)]
pub struct Shredders {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Data shredders of this event.
    #[must_use]
    pub fn shredders(&self) -> Shredders {
        Shredders {
            client: self.client(),
            path: self.path().join("shredders"),
        }
    }
}

impl Shredders {
    /// Lists the available shredders.
    pub fn list(&self, params: &ListParams) -> Request<Page<Shredder>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Starts exporting the data that the given shredders would delete.
    pub fn export(&self, body: &ExportShredders) -> Request<ShredderExport> {
        self.client
            .post(&self.path.join("export"))
            .json_body(body)
            .json()
    }

    /// Starts shredding the data of a finished export.
    ///
    /// Pending while the export is still running; failed if the export failed or
    /// timed out.
    pub fn shred(&self, export: &ShredderExport) -> Request<JobStatus<ShredJob>> {
        self.client
            .request_absolute(Method::POST, &export.shred)
            .job(decode::json)
    }

    /// Checks the result of a shredding process once.
    ///
    /// Use [`Request::wait`] to poll until it finishes.
    pub fn status(&self, job: &ShredJob) -> Request<JobStatus<ShredOutcome>> {
        self.client
            .request_absolute(Method::GET, &job.status)
            // Unlike other jobs, the shred status endpoint reports failures with `417`
            // and uses `410` for processes it no longer knows about.
            .job_with(
                JobCodes {
                    pending: &[StatusCode::CONFLICT],
                    failed: &[StatusCode::EXPECTATION_FAILED],
                },
                |raw| match raw.status {
                    StatusCode::GONE => Ok(ShredOutcome::Unknown),
                    _ => raw.ensure_success().map(|_| ShredOutcome::Completed),
                },
            )
    }
}
