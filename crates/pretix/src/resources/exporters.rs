//! Data exporters: bulk downloads of event or organizer data.
//!
//! Running an export is a two-step process: [`Exporters::run`] starts the export and
//! returns an [`ExportJob`] whose `download` link is polled until the file is ready
//! (see [`ExportJob::download_job`] and [`Client::download_job`]).
//!
//! The available exporters and their input parameters are **not** a stable API and
//! may change at any time, so export parameters are free-form.
//!
//! <https://docs.pretix.eu/dev/api/resources/exporters.html>

use serde::{Deserialize, Serialize};
use url::Url;

use crate::path::Path;
use crate::{Client, Download, EventScope, JobStatus, ListParams, OrganizerScope, Page, Request};

/// An export method.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Exporter {
    /// Identifier of the exporter, used with [`Exporters::run`].
    pub identifier: String,
    /// Human-readable name.
    pub verbose_name: String,
    /// The supported input fields. Exact types and validation rules are not given.
    #[serde(default)]
    pub input_parameters: Vec<ExporterParameter>,
}

/// An input field of an [`Exporter`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExporterParameter {
    /// Name of the field.
    pub name: String,
    /// Whether the field is required.
    pub required: bool,
    /// Valid values, for choice fields.
    #[serde(default)]
    pub choices: Option<Vec<serde_json::Value>>,
}

/// A started background job producing a file (an export or rendered tickets).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExportJob {
    /// Where to download the result. Responds with `409` while the job is running.
    pub download: Url,
}

impl ExportJob {
    /// Polls the result once. Equivalent to `client.download_job(&self.download)`.
    ///
    /// Use [`Request::wait`] to poll until the file is ready.
    pub fn download_job(&self, client: &Client) -> Request<JobStatus<Download>> {
        client.download_job(&self.download)
    }
}

/// Endpoints for the data exporters of an event or organizer. Created with
/// [`EventScope::exporters`] or [`OrganizerScope::exporters`].
#[derive(Debug, Clone)]
pub struct Exporters {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Data exporters of this event.
    #[must_use]
    pub fn exporters(&self) -> Exporters {
        Exporters {
            client: self.client(),
            path: self.path().join("exporters"),
        }
    }
}

impl OrganizerScope {
    /// Cross-event data exporters of this organizer.
    #[must_use]
    pub fn exporters(&self) -> Exporters {
        Exporters {
            client: self.client(),
            path: self.path().join("exporters"),
        }
    }
}

impl Exporters {
    /// Lists the available exporters and their input fields.
    pub fn list(&self, params: &ListParams) -> Request<Page<Exporter>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Starts an export. `params` are the exporter's input fields, e.g.
    /// `json!({"_format": "xlsx"})`.
    ///
    /// pretix asks clients not to run more than two exports at the same time.
    pub fn run<B: Serialize + ?Sized>(&self, identifier: &str, params: &B) -> Request<ExportJob> {
        self.client
            .post(&self.path.join(identifier).join("run"))
            .json_body(params)
            .json()
    }
}
