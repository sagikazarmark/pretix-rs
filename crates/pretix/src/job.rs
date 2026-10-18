use std::time::Duration;

use http::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::request::{BoxFuture, RawResponse, RequestBuilder};
use crate::{Client, Download, Error, Request, Result};

/// The state of a long-running server-side job (exports, ticket rendering, …).
///
/// Obtained by polling a job once. Use [`Request::wait`] to poll until the job
/// finishes, or poll manually (e.g. with durable timers between polls).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum JobStatus<T> {
    /// The job is queued or running (`409 Conflict`). Poll again later.
    Pending {
        /// Progress in percent, if known.
        percentage: Option<f64>,
    },
    /// The job finished.
    Ready(T),
    /// The job failed permanently (`410 Gone`).
    Failed {
        /// The error message, if provided.
        message: Option<String>,
    },
}

impl<T> JobStatus<T> {
    /// Whether the job is still running.
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    /// The result, if the job finished.
    pub fn ready(self) -> Option<T> {
        match self {
            Self::Ready(value) => Some(value),
            _ => None,
        }
    }
}

/// Options for [`Request::wait`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct PollOptions {
    /// Delay between polls. pretix asks clients to wait at least one second; shorter
    /// intervals are only meant for tests.
    pub interval: Duration,
    /// Give up after this long. `None` waits forever.
    pub timeout: Option<Duration>,
}

impl Default for PollOptions {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(2),
            timeout: Some(Duration::from_secs(600)),
        }
    }
}

impl PollOptions {
    /// Sets the delay between polls.
    #[must_use]
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Sets the timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }
}

#[derive(Deserialize)]
struct JobBody {
    #[serde(default)]
    percentage: Option<f64>,
    #[serde(default)]
    message: Option<String>,
}

/// The status codes a job endpoint uses to report its state.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JobCodes {
    /// Still running (body may contain `percentage`).
    pub(crate) pending: &'static [StatusCode],
    /// Failed permanently (body may contain `message`).
    pub(crate) failed: &'static [StatusCode],
}

impl JobCodes {
    /// `409 Conflict` while pending, `410 Gone` on failure (used by most endpoints).
    pub(crate) const DEFAULT: Self = Self {
        pending: &[StatusCode::CONFLICT],
        failed: &[StatusCode::GONE],
    };
}

impl RequestBuilder {
    /// Expects a job status response using [`JobCodes::DEFAULT`]. Other responses are
    /// decoded with `ready`.
    pub(crate) fn job<T: 'static>(
        self,
        ready: impl Fn(RawResponse) -> Result<T> + Send + Sync + 'static,
    ) -> Request<JobStatus<T>> {
        self.job_with(JobCodes::DEFAULT, ready)
    }

    /// Expects a job status response using custom status codes.
    pub(crate) fn job_with<T: 'static>(
        self,
        codes: JobCodes,
        ready: impl Fn(RawResponse) -> Result<T> + Send + Sync + 'static,
    ) -> Request<JobStatus<T>> {
        self.decode_with(move |raw| {
            let body = || serde_json::from_slice::<JobBody>(&raw.body).ok();
            if codes.pending.contains(&raw.status) {
                Ok(JobStatus::Pending {
                    percentage: body().and_then(|body| body.percentage),
                })
            } else if codes.failed.contains(&raw.status) {
                Ok(JobStatus::Failed {
                    message: body().and_then(|body| body.message),
                })
            } else {
                ready(raw).map(JobStatus::Ready)
            }
        })
    }

    /// Expects a file download that might not be ready yet (`409` while pending,
    /// `410` on failure).
    pub fn download_job(self) -> Request<JobStatus<Download>> {
        self.job(Download::from_response)
    }
}

impl Client {
    /// Polls a download link of a background job once (e.g. the `download` URL
    /// returned when running an export).
    pub fn download_job(&self, url: &Url) -> Request<JobStatus<Download>> {
        self.request_absolute(Method::GET, url).download_job()
    }
}

impl<T: Send + 'static> Request<JobStatus<T>> {
    /// Polls until the job finishes and returns its result.
    ///
    /// Fails with [`Error::JobFailed`] if the job fails and [`Error::JobTimeout`] if it
    /// does not finish in time.
    pub fn wait(self, options: PollOptions) -> BoxFuture<'static, Result<T>> {
        Box::pin(async move {
            let started = tokio::time::Instant::now();
            loop {
                match self.clone().await? {
                    JobStatus::Ready(value) => return Ok(value),
                    JobStatus::Failed { message } => {
                        return Err(Error::JobFailed(
                            message.unwrap_or_else(|| "no details given".into()),
                        ));
                    }
                    JobStatus::Pending { .. } => {}
                }
                // Checked after each poll, so slow responses count towards the timeout.
                if options
                    .timeout
                    .is_some_and(|timeout| started.elapsed() + options.interval > timeout)
                {
                    return Err(Error::JobTimeout);
                }
                tokio::time::sleep(options.interval).await;
            }
        })
    }
}
