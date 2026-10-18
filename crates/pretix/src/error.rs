use std::fmt;
use std::time::Duration;

use http::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

/// Errors returned by the client.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The API answered with a non-successful status code.
    #[error(transparent)]
    Api(Box<ApiError>),

    /// The request could not be sent or the response could not be received.
    #[error(transparent)]
    Transport(TransportError),

    /// The response body did not match the expected shape.
    #[error("failed to decode response body at `{path}`")]
    Decode {
        /// Path to the offending field (e.g. `results[3].positions[0].price`).
        path: String,
        /// The underlying JSON error.
        #[source]
        source: serde_json::Error,
    },

    /// The request could not be encoded (query string, body or header).
    #[error("failed to encode request: {0}")]
    Encode(String),

    /// The client is misconfigured.
    #[error("invalid client configuration: {0}")]
    Config(String),

    /// A background job reported a permanent failure.
    #[error("background job failed: {0}")]
    JobFailed(String),

    /// Waiting for a background job exceeded the configured timeout.
    #[error("timed out waiting for background job")]
    JobTimeout,

    /// The API returned a response that cannot be processed (e.g. a pagination link
    /// pointing back to an already fetched page).
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
}

impl Error {
    /// The HTTP status code, if the error originates from an API response.
    pub fn status(&self) -> Option<StatusCode> {
        self.as_api().map(ApiError::status)
    }

    /// The API error, if any.
    #[must_use]
    pub fn as_api(&self) -> Option<&ApiError> {
        match self {
            Self::Api(err) => Some(err),
            _ => None,
        }
    }

    /// Whether the failure is transient, i.e. the same request may succeed later.
    ///
    /// True for transport failures, for `408`, `429`, `500`, `502`, `503` and `504`
    /// responses, and for `409` responses with `Retry-After` (a concurrent request with
    /// the same idempotency key). Everything else (validation errors, missing
    /// permissions, business conflicts, …) will fail again.
    ///
    /// This does not consider whether repeating the request is *safe*; see
    /// [`Error::is_retryable`].
    #[must_use]
    pub fn is_transient(&self) -> bool {
        match self {
            Self::Api(err) => err.is_transient(),
            Self::Transport(err) => err.is_transient(),
            _ => false,
        }
    }

    /// Whether the request can be repeated safely and may then succeed.
    ///
    /// Requires a [transient](Error::is_transient) failure of a request that is safe to
    /// repeat: `GET`/`HEAD`/`OPTIONS` requests, requests with an idempotency key, and
    /// requests rejected with `429 Too Many Requests` (which were not executed).
    /// A mutating request without idempotency key that timed out may or may not have
    /// been executed, so it is not retryable.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Api(err) => err.is_retryable(),
            Self::Transport(err) => err.is_retryable(),
            _ => false,
        }
    }

    /// The delay requested by the server via `Retry-After`, if any.
    pub fn retry_after(&self) -> Option<Duration> {
        self.as_api().and_then(ApiError::retry_after)
    }

    pub(crate) fn decode(err: serde_path_to_error::Error<serde_json::Error>) -> Self {
        Self::Decode {
            path: err.path().to_string(),
            source: err.into_inner(),
        }
    }
}

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Self {
        Self::Api(Box::new(err))
    }
}

/// An error response returned by the pretix API.
///
/// pretix returns either `{"detail": "..."}` for general errors or an object mapping
/// field names to lists of messages for validation errors. Both are accessible here,
/// as is the raw body.
#[derive(Debug, Clone)]
pub struct ApiError {
    status: StatusCode,
    text: String,
    json: Option<Value>,
    retry_after: Option<Duration>,
    repeatable: bool,
}

impl ApiError {
    pub(crate) fn new(
        status: StatusCode,
        body: &[u8],
        retry_after: Option<Duration>,
        repeatable: bool,
    ) -> Self {
        Self {
            status,
            text: String::from_utf8_lossy(body).into_owned(),
            json: serde_json::from_slice(body).ok(),
            retry_after,
            repeatable,
        }
    }

    /// The HTTP status code.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// The `detail` message of general errors.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.json.as_ref()?.get("detail")?.as_str()
    }

    /// Field specific validation errors (`{"field": ["message", ...]}`).
    #[must_use]
    pub fn field_errors(&self) -> Option<&Map<String, Value>> {
        let object = self.json.as_ref()?.as_object()?;
        (!object.contains_key("detail")).then_some(object)
    }

    /// The response body parsed as JSON, if it was valid JSON.
    #[must_use]
    pub fn body(&self) -> Option<&Value> {
        self.json.as_ref()
    }

    /// The raw response body (lossily converted to UTF-8).
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Decodes the response body into a custom type.
    #[must_use]
    pub fn body_as<T: DeserializeOwned>(&self) -> Option<T> {
        serde_json::from_str(&self.text).ok()
    }

    /// The delay requested by the server via `Retry-After`, if any.
    #[must_use]
    pub fn retry_after(&self) -> Option<Duration> {
        self.retry_after
    }

    /// Whether the request can be repeated safely and may then succeed. See
    /// [`Error::is_retryable`].
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        self.is_transient() && (self.repeatable || self.status == StatusCode::TOO_MANY_REQUESTS)
    }

    /// Whether the same request may succeed later. See [`Error::is_transient`].
    #[must_use]
    pub fn is_transient(&self) -> bool {
        is_transient_status(self.status, self.retry_after)
    }
}

/// Whether a response status indicates a transient failure.
///
/// `409 Conflict` is only transient with `Retry-After`, which pretix sends while a
/// request with the same idempotency key is in progress; other conflicts (e.g.
/// insufficient gift card credit) are permanent.
pub(crate) fn is_transient_status(status: StatusCode, retry_after: Option<Duration>) -> bool {
    match status {
        StatusCode::CONFLICT => retry_after.is_some(),
        StatusCode::REQUEST_TIMEOUT
        | StatusCode::TOO_MANY_REQUESTS
        | StatusCode::INTERNAL_SERVER_ERROR
        | StatusCode::BAD_GATEWAY
        | StatusCode::SERVICE_UNAVAILABLE
        | StatusCode::GATEWAY_TIMEOUT => true,
        _ => false,
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pretix API returned {}", self.status)?;
        if let Some(detail) = self.detail() {
            return write!(f, ": {detail}");
        }
        let text = self.text.trim();
        if !text.is_empty() {
            const MAX: usize = 300;
            match text.char_indices().nth(MAX) {
                Some((idx, _)) => write!(f, ": {}…", &text[..idx])?,
                None => write!(f, ": {text}")?,
            }
        }
        Ok(())
    }
}

impl std::error::Error for ApiError {}

/// The error type of the HTTP client requests are sent through.
#[cfg(not(feature = "reqwest-middleware"))]
type HttpError = reqwest::Error;
/// The error type of the HTTP client requests are sent through.
#[cfg(feature = "reqwest-middleware")]
type HttpError = reqwest_middleware::Error;

/// A failure while sending a request or receiving a response.
#[derive(Debug, thiserror::Error)]
#[error("HTTP transport error ({}): {source}", self.kind())]
pub struct TransportError {
    #[source]
    source: HttpError,
    repeatable: bool,
}

impl TransportError {
    pub(crate) fn new(source: impl Into<HttpError>) -> Self {
        Self {
            source: source.into(),
            repeatable: false,
        }
    }

    pub(crate) fn with_repeatable(mut self, repeatable: bool) -> Self {
        self.repeatable = repeatable;
        self
    }

    fn kind(&self) -> &'static str {
        if self.is_middleware() {
            "middleware"
        } else if self.is_timeout() {
            "timeout"
        } else if self.is_connect() {
            "connect"
        } else if self.source.is_body() || self.source.is_decode() {
            "body"
        } else {
            "request"
        }
    }

    /// Whether the error was raised by a middleware rather than the HTTP client.
    #[cfg(feature = "reqwest-middleware")]
    #[must_use]
    pub fn is_middleware(&self) -> bool {
        self.source.is_middleware()
    }

    #[cfg(not(feature = "reqwest-middleware"))]
    #[allow(clippy::unused_self)]
    fn is_middleware(&self) -> bool {
        false
    }

    /// Whether the error is a timeout.
    #[must_use]
    pub fn is_timeout(&self) -> bool {
        self.source.is_timeout()
    }

    /// Whether the error happened while connecting.
    #[must_use]
    pub fn is_connect(&self) -> bool {
        self.source.is_connect()
    }

    /// Whether the same request may succeed later. See [`Error::is_transient`].
    ///
    /// Errors raised by middleware are opaque and therefore never considered transient.
    #[must_use]
    pub fn is_transient(&self) -> bool {
        !self.is_middleware() && !self.source.is_builder() && !self.source.is_redirect()
    }

    /// Whether the request can be repeated safely and may then succeed. See
    /// [`Error::is_retryable`].
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        self.is_transient() && self.repeatable
    }
}

impl From<TransportError> for Error {
    fn from(err: TransportError) -> Self {
        Self::Transport(err)
    }
}
