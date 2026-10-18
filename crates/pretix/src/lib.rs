//! An unofficial, async client for the [pretix](https://pretix.eu) REST API.
//!
//! # Quick start
//!
//! ```no_run
//! # async fn run() -> pretix::Result<()> {
//! use pretix::Client;
//! use pretix::resources::events::ListEvents;
//!
//! let client = Client::builder()
//!     .base_url("https://pretix.eu/api/v1/")
//!     .token("my-api-token")
//!     .build()?;
//!
//! let org = client.organizer("bigevents");
//!
//! // A single page, exactly as returned by the API.
//! let page = org.events().list(&ListEvents::default()).await?;
//! println!("{} events in total", page.count);
//!
//! // All results, following `next` links.
//! let events = org.events().list(&ListEvents::default()).all().await?;
//! # let _ = events;
//! # Ok(())
//! # }
//! ```
//!
//! # Design
//!
//! - Resources are reached through scopes that mirror the URL hierarchy:
//!   [`Client::organizer`] → [`OrganizerScope::event`] → resource handles such as
//!   `events()`, `orders()`, `items()`.
//! - Every endpoint method returns a [`Request`], which can be configured (e.g.
//!   [`Request::idempotency_key`]) and is then `.await`ed.
//! - List endpoints return a [`Page`]. [`Request::all`], [`Request::stream`] and
//!   [`Request::pages`] follow pagination links, and [`Client::page`] fetches a single
//!   page from a serializable [`PageCursor`].
//! - Long-running server jobs (exports, ticket rendering, …) return a [`JobStatus`] that
//!   can be polled once, or awaited with [`Request::wait`].
//!
//! All futures and streams are `Send`, and all response types implement both
//! `Serialize` and `Deserialize` so they can be journaled by durable execution engines.
//!
//! # Features
//!
//! - `rustls` *(default)*: TLS via rustls.
//! - `native-tls`: TLS via the platform implementation.
//! - `plugins`: endpoints of pretix plugins that are not bundled with pretix.
//! - `hosted`: endpoints only available on pretix Hosted.
//! - `full`: everything above except TLS selection.
//! - `schemars`: implements `schemars::JsonSchema` for every serializable type (responses,
//!   query parameters and request bodies), e.g. for Restate handler inputs and outputs.
//!   The schemas describe the JSON (de)serialization: decimals are strings, enum-like
//!   fields list their known values but accept any string.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::struct_excessive_bools,
    reason = "models mirror pretix API objects, which carry many boolean flags"
)]

mod auth;
mod client;
mod conditional;
mod download;
mod error;
mod job;
mod pagination;
mod path;
pub mod query;
mod request;
pub mod resources;
mod retry;
mod scope;
mod serde_util;
pub mod types;
mod upload;

pub use auth::Auth;
pub use client::{Client, ClientBuilder};
pub use conditional::Conditional;
pub use download::Download;
pub use error::{ApiError, Error, TransportError};
pub use job::{JobStatus, PollOptions};
pub use pagination::{Page, PageCursor, ResponseMeta};
pub use query::ListParams;
pub use request::{Request, RequestBuilder};
pub use retry::RetryPolicy;
pub use scope::{EventScope, OrganizerScope};

/// Re-exported so callers can name the types used in public APIs.
pub use http;
/// Re-exported so callers can name the types used in public APIs.
pub use jiff;
/// Re-exported so callers can name the types used in public APIs.
pub use rust_decimal;
/// Re-exported so callers can name the types used in public APIs.
pub use url;

/// A `Result` alias with [`Error`] as the default error type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Compiles the README examples as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
