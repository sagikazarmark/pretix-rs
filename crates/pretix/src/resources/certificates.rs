//! Certificates of attendance (plugin `pretix-certificates`).
//!
//! Downloading a certificate is a two-step process: [`Certificates::start`] makes pretix
//! generate the PDF and returns a [`CertificateJob`], which is then polled with
//! [`CertificateJob::download_job`] (or awaited with [`Request::wait`]).
//!
//! <https://docs.pretix.eu/dev/api/resources/certificates.html>

use http::StatusCode;
use http::header::LOCATION;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::path::Path;
use crate::{Client, Download, EventScope, JobStatus, Request};

/// A certificate being generated, returned by [`Certificates::start`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CertificateJob {
    /// The URL to poll for the PDF (contains the `result` ID).
    pub download: Url,
}

impl CertificateJob {
    /// Polls the certificate once. Shortcut for [`Client::download_job`].
    pub fn download_job(&self, client: &Client) -> Request<JobStatus<Download>> {
        client.download_job(&self.download)
    }
}

/// Endpoints for certificates of attendance. Created with [`EventScope::certificates`].
#[derive(Debug, Clone)]
pub struct Certificates {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Certificates of attendance of this event's order positions.
    #[must_use]
    pub fn certificates(&self) -> Certificates {
        Certificates {
            client: self.client(),
            path: self.path().join("orderpositions"),
        }
    }
}

impl Certificates {
    /// Starts generating the certificate PDF of an order position.
    ///
    /// pretix answers with `303 See Other` pointing at the result URL. Whether or not
    /// the HTTP client follows the redirect, the result URL is returned.
    pub fn start(&self, position_id: u64) -> Request<CertificateJob> {
        self.client
            .get(&self.path.join(position_id).join("certificate"))
            .decode_with(|raw| {
                let download = if raw.status == StatusCode::SEE_OTHER {
                    raw.header_str(LOCATION)
                        .and_then(|location| raw.url.join(location).ok())
                } else if raw.status.is_success() || raw.status == StatusCode::CONFLICT {
                    // The redirect was followed; the final URL is the result URL.
                    Some(raw.url.clone())
                } else {
                    None
                };
                match download {
                    Some(download) => Ok(CertificateJob { download }),
                    None => Err(raw.into_error()),
                }
            })
    }
}
