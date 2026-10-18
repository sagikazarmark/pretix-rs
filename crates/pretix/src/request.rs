use std::fmt;
use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http::header::{CONTENT_TYPE, HeaderName};
use http::{HeaderMap, HeaderValue, Method, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use crate::{ApiError, Client, Error, Result};

pub(crate) const IDEMPOTENCY_KEY: HeaderName = HeaderName::from_static("x-idempotency-key");

/// A boxed, `Send` future.
pub(crate) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Turns a raw response into the typed result of a request.
pub(crate) type Decoder<T> = Arc<dyn Fn(RawResponse) -> Result<T> + Send + Sync>;

/// A buffered HTTP response.
#[derive(Debug, Clone)]
pub(crate) struct RawResponse {
    /// The final URL, after following redirects.
    #[cfg_attr(not(feature = "plugins"), allow(dead_code))]
    pub(crate) url: Url,
    pub(crate) status: StatusCode,
    pub(crate) headers: HeaderMap,
    pub(crate) body: Bytes,
    /// Whether the request is safe to repeat (see [`Error::is_retryable`]).
    pub(crate) repeatable: bool,
}

impl RawResponse {
    pub(crate) fn retry_after(&self) -> Option<Duration> {
        crate::client::parse_retry_after(&self.headers)
    }

    pub(crate) fn into_error(self) -> Error {
        let retry_after = self.retry_after();
        ApiError::new(self.status, &self.body, retry_after, self.repeatable).into()
    }

    /// Returns an [`Error::Api`] unless the status is `2xx`.
    pub(crate) fn ensure_success(self) -> Result<Self> {
        if self.status.is_success() {
            Ok(self)
        } else {
            Err(self.into_error())
        }
    }

    pub(crate) fn json<T: DeserializeOwned>(&self) -> Result<T> {
        decode_json(&self.body)
    }

    pub(crate) fn header_str(&self, name: impl http::header::AsHeaderName) -> Option<&str> {
        self.headers.get(name)?.to_str().ok()
    }
}

pub(crate) fn decode_json<T: DeserializeOwned>(body: &[u8]) -> Result<T> {
    let deserializer = &mut serde_json::Deserializer::from_slice(body);
    serde_path_to_error::deserialize(deserializer).map_err(Error::decode)
}

/// Shared decoders. Every endpoint picks one of these (or composes them).
pub(crate) mod decode {
    use super::{DeserializeOwned, RawResponse, Result};

    pub(crate) fn json<T: DeserializeOwned>(raw: RawResponse) -> Result<T> {
        raw.ensure_success()?.json()
    }

    pub(crate) fn empty(raw: RawResponse) -> Result<()> {
        raw.ensure_success().map(drop)
    }
}

/// Everything needed to send a request (and send it again when retrying or polling).
#[derive(Debug, Clone)]
pub(crate) struct RequestParts {
    pub(crate) method: Method,
    pub(crate) url: Url,
    pub(crate) headers: HeaderMap,
    pub(crate) body: Option<Bytes>,
    /// Errors that occurred while building the request are reported when it is sent,
    /// so that builder methods do not need to return `Result`.
    pub(crate) deferred_error: Option<String>,
}

/// Builder for requests to arbitrary endpoints, created with [`Client::request`].
///
/// Resource handles use the same builder internally. Finish it with a method that
/// determines how the response is decoded (e.g. [`RequestBuilder::json`]).
#[must_use = "finish the builder, e.g. with `.json()`, and `.await` the request"]
pub struct RequestBuilder {
    client: Client,
    parts: RequestParts,
}

impl fmt::Debug for RequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequestBuilder")
            .field("method", &self.parts.method)
            .field("url", &self.parts.url.as_str())
            .finish_non_exhaustive()
    }
}

impl RequestBuilder {
    pub(crate) fn new(client: Client, method: Method, url: Url) -> Self {
        Self {
            client,
            parts: RequestParts {
                method,
                url,
                headers: HeaderMap::new(),
                body: None,
                deferred_error: None,
            },
        }
    }

    fn defer_error(&mut self, message: String) {
        self.parts.deferred_error.get_or_insert(message);
    }

    /// Makes the request fail with `message` when sent.
    pub(crate) fn fail(mut self, message: String) -> Self {
        self.defer_error(message);
        self
    }

    /// Appends query parameters serialized from `params` (a struct or map; sequences
    /// produce repeated keys).
    pub fn query<Q: Serialize + ?Sized>(mut self, params: &Q) -> Self {
        match serde_html_form::to_string(params) {
            Ok(encoded) if encoded.is_empty() => {}
            Ok(encoded) => {
                let query = match self.parts.url.query() {
                    Some(existing) if !existing.is_empty() => format!("{existing}&{encoded}"),
                    _ => encoded,
                };
                self.parts.url.set_query(Some(&query));
            }
            Err(err) => self.defer_error(format!("query string: {err}")),
        }
        self
    }

    /// Appends a single query parameter.
    pub(crate) fn query_pair(mut self, key: &str, value: &str) -> Self {
        self.parts.url.query_pairs_mut().append_pair(key, value);
        self
    }

    /// Sets a JSON request body.
    pub fn json_body<B: Serialize + ?Sized>(mut self, body: &B) -> Self {
        match serde_json::to_vec(body) {
            Ok(bytes) => {
                self.parts.body = Some(bytes.into());
                self.parts
                    .headers
                    .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            }
            Err(err) => self.defer_error(format!("request body: {err}")),
        }
        self
    }

    /// Sets a raw request body.
    pub fn body(mut self, body: Bytes, content_type: &str) -> Self {
        self.parts.body = Some(body);
        self.header(CONTENT_TYPE, content_type)
    }

    /// Sets a request header.
    pub fn header(mut self, name: HeaderName, value: &str) -> Self {
        match HeaderValue::try_from(value) {
            Ok(value) => {
                self.parts.headers.insert(name, value);
            }
            Err(_) => self.defer_error(format!("invalid value for header `{name}`")),
        }
        self
    }

    /// Finishes the request with a custom decoder.
    pub(crate) fn decode_with<T, F>(self, decode: F) -> Request<T>
    where
        F: Fn(RawResponse) -> Result<T> + Send + Sync + 'static,
    {
        Request {
            client: self.client.clone(),
            parts: self.parts,
            decode: Arc::new(decode),
        }
    }

    /// Expects a `2xx` response with a JSON body.
    pub fn json<T: DeserializeOwned + 'static>(self) -> Request<T> {
        self.decode_with(decode::json::<T>)
    }

    /// Expects a `2xx` response and ignores the body.
    pub fn empty(self) -> Request<()> {
        self.decode_with(decode::empty)
    }
}

/// A prepared API request.
///
/// Requests are lazy: nothing is sent until the request is `.await`ed. Before that,
/// it can be configured, e.g. with [`Request::idempotency_key`].
///
/// The future returned by `.await` is `Send`. A request can be cloned to send it again.
#[must_use = "requests do nothing unless `.await`ed"]
pub struct Request<T> {
    pub(crate) client: Client,
    pub(crate) parts: RequestParts,
    pub(crate) decode: Decoder<T>,
}

impl<T> Clone for Request<T> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            parts: self.parts.clone(),
            decode: Arc::clone(&self.decode),
        }
    }
}

impl<T> fmt::Debug for Request<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request")
            .field("method", &self.parts.method)
            .field("url", &self.parts.url.as_str())
            .finish_non_exhaustive()
    }
}

impl<T> Request<T> {
    /// The HTTP method.
    pub fn method(&self) -> &Method {
        &self.parts.method
    }

    /// The full request URL, including the query string.
    pub fn url(&self) -> &Url {
        &self.parts.url
    }

    /// Sets the `X-Idempotency-Key` header.
    ///
    /// pretix will execute a mutating request at most once per key (for 24 hours) and
    /// replay the original response for repeated requests. Requests with a key are
    /// also eligible for automatic retries (see [`RetryPolicy`](crate::RetryPolicy)).
    ///
    /// The key should be unique per logical operation; a random UUID is recommended.
    /// Durable execution engines should derive it deterministically.
    pub fn idempotency_key(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        match HeaderValue::try_from(key) {
            Ok(value) => {
                self.parts.headers.insert(IDEMPOTENCY_KEY, value);
            }
            Err(_) => {
                self.parts
                    .deferred_error
                    .get_or_insert_with(|| "invalid idempotency key".into());
            }
        }
        self
    }

    /// Decodes a successful response body into a custom type instead.
    ///
    /// Useful for fields not (yet) modeled by this crate, e.g. `decode_as::<serde_json::Value>()`.
    pub fn decode_as<U: DeserializeOwned + 'static>(self) -> Request<U> {
        Request {
            client: self.client.clone(),
            parts: self.parts,
            decode: Arc::new(decode::json::<U>),
        }
    }

    /// Sends the request. Equivalent to `.await`ing it.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the request cannot be sent, pretix responds with an error
    /// status, or the response body cannot be decoded.
    pub async fn send(self) -> Result<T>
    where
        T: Send + 'static,
    {
        self.await
    }

    pub(crate) fn map_decoder<U>(self, wrap: impl FnOnce(Decoder<T>) -> Decoder<U>) -> Request<U> {
        Request {
            client: self.client.clone(),
            parts: self.parts,
            decode: wrap(self.decode),
        }
    }

    pub(crate) fn with_header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.parts.headers.insert(name, value);
        self
    }
}

impl<T: Send + 'static> IntoFuture for Request<T> {
    type Output = Result<T>;
    type IntoFuture = BoxFuture<'static, Result<T>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let raw = self.client.execute(&self.parts).await?;
            (self.decode)(raw)
        })
    }
}
