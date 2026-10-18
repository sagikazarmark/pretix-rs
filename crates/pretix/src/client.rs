use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http::header::{ACCEPT, AUTHORIZATION, RETRY_AFTER, USER_AGENT};
use http::{HeaderMap, HeaderValue, Method};
use url::Url;

use crate::error::TransportError;
use crate::path::Path;
use crate::request::{IDEMPOTENCY_KEY, RawResponse, RequestBuilder, RequestParts};
use crate::retry::{Outcome, OutcomeResult, RequestKind};
use crate::{Auth, Error, Result, RetryPolicy};

const DEFAULT_BASE_URL: &str = "https://pretix.eu/api/v1/";
const DEFAULT_USER_AGENT: &str = concat!("pretix-rs/", env!("CARGO_PKG_VERSION"));
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// A client for the pretix REST API.
///
/// Cloning is cheap: clones share the connection pool and configuration.
#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

struct Inner {
    http: reqwest::Client,
    base_url: Url,
    authorization: HeaderValue,
    user_agent: HeaderValue,
    retry: RetryPolicy,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.inner.base_url.as_str())
            .field("retry", &self.inner.retry)
            .finish_non_exhaustive()
    }
}

impl Client {
    /// Creates a builder.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// The API root all requests are relative to (e.g. `https://pretix.eu/api/v1/`).
    #[must_use]
    pub fn base_url(&self) -> &Url {
        &self.inner.base_url
    }

    pub(crate) fn get(&self, path: &Path) -> RequestBuilder {
        self.request_path(Method::GET, path)
    }

    pub(crate) fn post(&self, path: &Path) -> RequestBuilder {
        self.request_path(Method::POST, path)
    }

    pub(crate) fn patch(&self, path: &Path) -> RequestBuilder {
        self.request_path(Method::PATCH, path)
    }

    pub(crate) fn delete(&self, path: &Path) -> RequestBuilder {
        self.request_path(Method::DELETE, path)
    }

    /// Builds a request to any endpoint, e.g. one not modeled by this crate.
    ///
    /// `path` is relative to the base URL (`organizers/acme/events/`); remember the
    /// trailing slash pretix expects, and percent-encode user input. Absolute URLs
    /// returned by the API are accepted too, and are sent to the configured origin.
    ///
    /// ```no_run
    /// # async fn run(client: pretix::Client) -> pretix::Result<()> {
    /// use pretix::http::Method;
    ///
    /// let value: serde_json::Value = client
    ///     .request(Method::GET, "organizers/acme/my_plugin_endpoint/")
    ///     .json()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn request(&self, method: Method, path: &str) -> RequestBuilder {
        if let Ok(url) = Url::parse(path) {
            return self.request_absolute(method, &url);
        }
        match self.inner.base_url.join(path.trim_start_matches('/')) {
            Ok(url) => RequestBuilder::new(self.clone(), method, url),
            Err(err) => RequestBuilder::new(self.clone(), method, self.inner.base_url.clone())
                .fail(format!("invalid path `{path}`: {err}")),
        }
    }

    pub(crate) fn request_path(&self, method: Method, path: &Path) -> RequestBuilder {
        match path.to_url(&self.inner.base_url) {
            Ok(url) => RequestBuilder::new(self.clone(), method, url),
            Err(err) => {
                RequestBuilder::new(self.clone(), method, self.inner.base_url.clone()).fail(err)
            }
        }
    }

    /// Builds a request for an absolute URL returned by the API (pagination or
    /// download links). The URL is moved onto the configured base URL's origin so that
    /// credentials are never sent elsewhere.
    pub(crate) fn request_absolute(&self, method: Method, url: &Url) -> RequestBuilder {
        RequestBuilder::new(self.clone(), method, self.resolve(url))
    }

    /// Where a request to an absolute URL returned by the API is actually sent.
    pub(crate) fn resolve(&self, url: &Url) -> Url {
        crate::path::rebase(&self.inner.base_url, url)
    }

    pub(crate) async fn execute(&self, parts: &RequestParts) -> Result<RawResponse> {
        if let Some(err) = &parts.deferred_error {
            return Err(Error::Encode(err.clone()));
        }
        let kind = RequestKind {
            method: &parts.method,
            has_idempotency_key: parts.headers.contains_key(IDEMPOTENCY_KEY),
        };
        let repeatable = kind.is_repeatable();
        let mut attempt = 0;
        loop {
            let result = self.send_once(parts, repeatable).await;
            let outcome = match &result {
                Ok(raw) if raw.status.is_success() || raw.status.is_redirection() => {
                    return result.map_err(Error::from);
                }
                Ok(raw) => OutcomeResult::Status {
                    status: raw.status,
                    retry_after: raw.retry_after(),
                },
                Err(err) if err.is_transient() => OutcomeResult::Transport,
                Err(_) => return result.map_err(Error::from),
            };
            let outcome = Outcome {
                request: kind,
                result: outcome,
            };
            match self.inner.retry.delay(attempt, outcome) {
                Some(delay) => {
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
                None => return result.map_err(Error::from),
            }
        }
    }

    async fn send_once(
        &self,
        parts: &RequestParts,
        repeatable: bool,
    ) -> std::result::Result<RawResponse, TransportError> {
        let mut headers = HeaderMap::with_capacity(parts.headers.len() + 3);
        headers.insert(AUTHORIZATION, self.inner.authorization.clone());
        headers.insert(USER_AGENT, self.inner.user_agent.clone());
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        for (name, value) in &parts.headers {
            headers.insert(name, value.clone());
        }

        let mut request = self
            .inner
            .http
            .request(parts.method.clone(), parts.url.clone())
            .headers(headers);
        if let Some(body) = &parts.body {
            request = request.body(body.clone());
        }

        let transport = |err| TransportError::new(err).with_repeatable(repeatable);
        let response = request.send().await.map_err(transport)?;
        let url = response.url().clone();
        let status = response.status();
        let headers = response.headers().clone();
        let body: Bytes = response.bytes().await.map_err(transport)?;
        Ok(RawResponse {
            url,
            status,
            headers,
            body,
            repeatable,
        })
    }
}

/// Parses a `Retry-After` header (delay in seconds or an HTTP date).
pub(crate) fn parse_retry_after(headers: &HeaderMap) -> Option<Duration> {
    let value = headers.get(RETRY_AFTER)?.to_str().ok()?.trim();
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    let at = jiff::fmt::rfc2822::parse(value).ok()?.timestamp();
    let now = jiff::Timestamp::now();
    let span = at.duration_since(now);
    Some(Duration::try_from(span).unwrap_or(Duration::ZERO))
}

/// Builder for [`Client`].
#[derive(Debug, Default)]
#[must_use]
pub struct ClientBuilder {
    base_url: Option<String>,
    auth: Option<Auth>,
    retry: Option<RetryPolicy>,
    http: Option<reqwest::Client>,
    user_agent: Option<String>,
    timeout: Option<Duration>,
}

impl ClientBuilder {
    /// Sets the API root. Defaults to `https://pretix.eu/api/v1/`.
    ///
    /// For self-hosted instances pass e.g. `https://tickets.example.com/api/v1/`. If
    /// only an origin is given (`https://tickets.example.com`), `/api/v1/` is appended.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Authenticates with a team API token.
    pub fn token(self, token: impl Into<String>) -> Self {
        self.auth(Auth::token(token))
    }

    /// Sets the credentials.
    pub fn auth(mut self, auth: Auth) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Sets the retry policy. Defaults to [`RetryPolicy::default`].
    pub fn retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry = Some(policy);
        self
    }

    /// Uses a preconfigured HTTP client (proxies, custom TLS, …).
    ///
    /// Cannot be combined with [`ClientBuilder::timeout`]; configure the timeout on the
    /// HTTP client instead. The client should not follow redirects
    /// (`reqwest::redirect::Policy::none()`): redirected `POST`s are re-sent as `GET`s
    /// by HTTP clients, which could make failed actions look successful.
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http = Some(client);
        self
    }

    /// Overrides the `User-Agent` header.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Sets the total request timeout of the default HTTP client. Defaults to 60 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Builds the client.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if the base URL, API token, or user agent is invalid,
    /// no credentials are configured, a timeout is combined with a custom HTTP client,
    /// or the default HTTP client cannot be built.
    pub fn build(self) -> Result<Client> {
        let base_url = parse_base_url(self.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL))?;
        let auth = self
            .auth
            .ok_or_else(|| Error::Config("no credentials configured".into()))?;
        let user_agent =
            HeaderValue::try_from(self.user_agent.as_deref().unwrap_or(DEFAULT_USER_AGENT))
                .map_err(|_| Error::Config("invalid user agent".into()))?;
        let http = match (self.http, self.timeout) {
            (Some(_), Some(_)) => {
                return Err(Error::Config(
                    "`timeout` cannot be combined with a custom `http_client`".into(),
                ));
            }
            (Some(http), None) => http,
            (None, timeout) => reqwest::Client::builder()
                .timeout(timeout.unwrap_or(DEFAULT_TIMEOUT))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|err| Error::Config(format!("failed to build HTTP client: {err}")))?,
        };
        Ok(Client {
            inner: Arc::new(Inner {
                http,
                base_url,
                authorization: auth.header_value()?,
                user_agent,
                retry: self.retry.unwrap_or_default(),
            }),
        })
    }
}

fn parse_base_url(raw: &str) -> Result<Url> {
    let mut url =
        Url::parse(raw).map_err(|err| Error::Config(format!("invalid base URL `{raw}`: {err}")))?;
    if url.cannot_be_a_base() || !matches!(url.scheme(), "http" | "https") {
        return Err(Error::Config(format!(
            "invalid base URL `{raw}`: must be http(s)"
        )));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(Error::Config(
            "the base URL must not contain credentials; use `token` instead".into(),
        ));
    }
    url.set_query(None);
    url.set_fragment(None);
    if url.path() == "/" {
        url.set_path("/api/v1/");
    } else if !url.path().ends_with('/') {
        let path = format!("{}/", url.path());
        url.set_path(&path);
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_base_url() {
        assert_eq!(
            parse_base_url("https://tickets.example.com")
                .unwrap()
                .as_str(),
            "https://tickets.example.com/api/v1/"
        );
        assert_eq!(
            parse_base_url("https://example.com/pretix/api/v1")
                .unwrap()
                .as_str(),
            "https://example.com/pretix/api/v1/"
        );
        assert!(parse_base_url("mailto:foo@example.com").is_err());
        assert!(parse_base_url("https://user:pass@example.com/api/v1/").is_err());
    }

    #[test]
    fn rejects_timeout_with_custom_http_client() {
        let result = Client::builder()
            .token("t")
            .timeout(Duration::from_secs(1))
            .http_client(reqwest::Client::new())
            .build();
        assert!(matches!(result, Err(Error::Config(_))));
    }

    #[test]
    fn requires_credentials() {
        assert!(matches!(Client::builder().build(), Err(Error::Config(_))));
    }

    #[test]
    fn parses_retry_after_seconds() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("7"));
        assert_eq!(parse_retry_after(&headers), Some(Duration::from_secs(7)));
    }

    #[test]
    fn debug_does_not_leak_token() {
        let client = Client::builder().token("supersecret").build().unwrap();
        assert!(!format!("{client:?}").contains("supersecret"));
    }
}
