use std::sync::Arc;

use http::header::IF_MODIFIED_SINCE;
use http::{HeaderValue, StatusCode};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::Request;

/// The result of a conditional request (see [`Request::if_modified_since`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Conditional<T> {
    /// Nothing changed since the given time (`304 Not Modified`).
    NotModified,
    /// The resource changed; the full, current response.
    Modified(T),
}

impl<T> Conditional<T> {
    /// Whether the resource changed.
    pub fn is_modified(&self) -> bool {
        matches!(self, Self::Modified(_))
    }

    /// The response, if the resource changed.
    pub fn modified(self) -> Option<T> {
        match self {
            Self::Modified(value) => Some(value),
            Self::NotModified => None,
        }
    }
}

impl<T: 'static> Request<T> {
    /// Only fetches the resource if it changed after `since` (`If-Modified-Since`).
    ///
    /// Typically `since` is [`ResponseMeta::last_modified`](crate::ResponseMeta) of a
    /// previous response. pretix supports this on the lists of item categories, items,
    /// questions, quotas, sub-events and tax rules; other endpoints always answer with
    /// [`Conditional::Modified`].
    pub fn if_modified_since(self, since: Timestamp) -> Request<Conditional<T>> {
        let value = jiff::fmt::rfc2822::DateTimePrinter::new()
            .timestamp_to_rfc9110_string(&since)
            .ok()
            .and_then(|value| HeaderValue::try_from(value).ok());
        let mut request = match value {
            Some(value) => self.with_header(IF_MODIFIED_SINCE, value),
            None => self,
        };
        if request.parts.headers.get(IF_MODIFIED_SINCE).is_none() {
            request
                .parts
                .deferred_error
                .get_or_insert_with(|| "cannot format If-Modified-Since timestamp".into());
        }
        request.map_decoder(|decode| {
            Arc::new(move |raw| {
                if raw.status == StatusCode::NOT_MODIFIED {
                    Ok(Conditional::NotModified)
                } else {
                    decode(raw).map(Conditional::Modified)
                }
            })
        })
    }
}
