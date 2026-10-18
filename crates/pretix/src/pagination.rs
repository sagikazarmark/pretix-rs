use std::collections::HashSet;

use futures_util::stream::{self, BoxStream, StreamExt, TryStreamExt};
use http::Method;
use jiff::Timestamp;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::request::{BoxFuture, RawResponse};
use crate::{Client, Error, Request, Result};

/// One page of a paginated list, exactly as returned by the API.
///
/// Use [`Request::all`], [`Request::stream`] or [`Request::pages`] to follow pagination
/// links automatically, or [`Page::next_cursor`] with [`Client::page`] to do it
/// step by step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct Page<T> {
    /// Total number of results across all pages.
    #[serde(default)]
    pub count: u64,
    /// Link to the next page.
    pub next: Option<Url>,
    /// Link to the previous page.
    pub previous: Option<Url>,
    /// The results on this page.
    pub results: Vec<T>,
    /// Metadata from the response headers.
    #[serde(default)]
    pub meta: ResponseMeta,
}

impl<T> Page<T> {
    /// Creates a single, complete page of results (e.g. for tests).
    #[must_use]
    pub fn new(results: Vec<T>) -> Self {
        Self {
            count: results.len() as u64,
            next: None,
            previous: None,
            results,
            meta: ResponseMeta::default(),
        }
    }

    /// A cursor for the next page, if there is one.
    pub fn next_cursor(&self) -> Option<PageCursor> {
        self.next.clone().map(PageCursor)
    }

    /// A cursor for the previous page, if there is one.
    pub fn previous_cursor(&self) -> Option<PageCursor> {
        self.previous.clone().map(PageCursor)
    }

    /// Whether this is the last page.
    #[must_use]
    pub fn is_last(&self) -> bool {
        self.next.is_none()
    }
}

impl<T> IntoIterator for Page<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.results.into_iter()
    }
}

/// Response metadata taken from HTTP headers.
///
/// When following pagination ([`Request::all`], [`Request::stream`]), use the metadata
/// of the *first* page for incremental synchronization: it was generated before any
/// later page.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ResponseMeta {
    /// The `X-Page-Generated` header: the server time when the list was generated.
    ///
    /// Pass it as `modified_since` on the next request to only receive changed
    /// objects (supported by orders).
    pub page_generated: Option<Timestamp>,
    /// The `Last-Modified` header: the last modification of any object in the list.
    ///
    /// Pass it to [`Request::if_modified_since`] on the next request.
    pub last_modified: Option<Timestamp>,
}

impl ResponseMeta {
    pub(crate) fn from_response(raw: &RawResponse) -> Self {
        Self {
            page_generated: raw
                .header_str("x-page-generated")
                .and_then(|value| value.parse().ok()),
            last_modified: raw
                .header_str(http::header::LAST_MODIFIED)
                .and_then(|value| jiff::fmt::rfc2822::parse(value).ok())
                .map(|zoned| zoned.timestamp()),
        }
    }
}

/// A serializable reference to a page of results (the `next`/`previous` link).
///
/// Fetch it with [`Client::page`]. Cursors can be persisted, e.g. to process a large
/// list one page at a time in a durable workflow.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct PageCursor(Url);

impl PageCursor {
    /// Creates a cursor from a page link.
    #[must_use]
    pub fn new(url: Url) -> Self {
        Self(url)
    }

    /// The page link.
    #[must_use]
    pub fn url(&self) -> &Url {
        &self.0
    }
}

impl Client {
    /// Fetches the page a cursor points to.
    ///
    /// The link is resolved against the configured base URL's origin, so credentials
    /// are never sent to a different host.
    pub fn page<T: DeserializeOwned + 'static>(&self, cursor: &PageCursor) -> Request<Page<T>> {
        self.request_absolute(Method::GET, &cursor.0).page()
    }
}

impl crate::request::RequestBuilder {
    /// Expects a paginated list response.
    pub fn page<T: DeserializeOwned + 'static>(self) -> Request<Page<T>> {
        self.decode_with(|raw| {
            let raw = raw.ensure_success()?;
            let mut page: Page<T> = raw.json()?;
            page.meta = ResponseMeta::from_response(&raw);
            Ok(page)
        })
    }
}

enum Next<T> {
    First(Request<Page<T>>),
    Cursor(PageCursor, HashSet<Url>),
}

/// Upper bound for preallocating results, so that a bogus `count` cannot cause huge
/// allocations.
const MAX_PREALLOCATION: u64 = 10_000;

impl<T> Request<Page<T>>
where
    T: DeserializeOwned + Send + 'static,
{
    /// Streams all pages, starting with this request and following `next` links.
    pub fn pages(self) -> BoxStream<'static, Result<Page<T>>> {
        let client = self.client.clone();
        stream::try_unfold(Some(Next::First(self)), move |next| {
            let client = client.clone();
            async move {
                let (page, mut seen) = match next {
                    None => return Ok(None),
                    Some(Next::First(request)) => {
                        let mut seen = HashSet::new();
                        seen.insert(request.url().clone());
                        (request.await?, seen)
                    }
                    Some(Next::Cursor(cursor, seen)) => (client.page::<T>(&cursor).await?, seen),
                };
                let following = match page.next_cursor() {
                    // An empty page cannot be followed by more results.
                    Some(_) if page.results.is_empty() => None,
                    Some(cursor) => {
                        let url = client.resolve(cursor.url());
                        if !seen.insert(url.clone()) {
                            return Err(Error::UnexpectedResponse(format!(
                                "pagination link {url} points to an already fetched page"
                            )));
                        }
                        Some(Next::Cursor(cursor, seen))
                    }
                    None => None,
                };
                Ok(Some((page, following)))
            }
        })
        .boxed()
    }

    /// Streams all results across all pages.
    pub fn stream(self) -> BoxStream<'static, Result<T>> {
        self.pages()
            .map_ok(|page| stream::iter(page.results.into_iter().map(Ok)))
            .try_flatten()
            .boxed()
    }

    /// Collects all results across all pages.
    pub fn all(self) -> BoxFuture<'static, Result<Vec<T>>> {
        Box::pin(async move {
            let mut pages = self.pages();
            let mut results = Vec::new();
            while let Some(page) = pages.try_next().await? {
                if results.is_empty() {
                    let expected = page.count.min(MAX_PREALLOCATION);
                    results.reserve(usize::try_from(expected).unwrap_or(0));
                }
                results.extend(page.results);
            }
            Ok(results)
        })
    }
}
