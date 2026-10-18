//! PDF ticket output: ticket layouts and batch rendering of tickets.
//!
//! <https://docs.pretix.eu/dev/api/resources/ticketoutputpdf.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::resources::exporters::ExportJob;
use crate::types::FileId;
use crate::{Client, EventScope, ListParams, Page, Request};

/// A PDF ticket layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TicketLayout {
    /// Internal layout ID.
    pub id: u64,
    /// Internal description.
    pub name: String,
    /// Whether this is the default layout.
    pub default: bool,
    /// Dynamic layout specification, following pretix' `pdf-layout.schema.json`.
    pub layout: serde_json::Value,
    /// URL of the background PDF file.
    pub background: Option<String>,
    /// Products this layout is assigned to.
    #[serde(default)]
    pub item_assignments: Vec<TicketLayoutAssignment>,
}

/// An assignment of a product to a [`TicketLayout`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TicketLayoutAssignment {
    /// Sales channel (defaults to `web`).
    pub sales_channel: String,
    /// Item ID.
    pub item: u64,
}

/// An assignment of a product to a layout, as listed by [`TicketLayoutItems::list`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TicketLayoutItem {
    /// Internal ID of the assignment.
    pub id: u64,
    /// Layout ID.
    pub layout: u64,
    /// Item ID.
    pub item: u64,
    /// Sales channel.
    pub sales_channel: String,
}

/// Body of [`TicketLayouts::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateTicketLayout {
    /// Internal description.
    #[builder(into)]
    pub name: String,
    /// Whether this is the default layout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
    /// Dynamic layout specification, following pretix' `pdf-layout.schema.json`.
    pub layout: serde_json::Value,
    /// Background PDF uploaded with [`Client::upload`](crate::Client::upload). pretix'
    /// default background is used if unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<FileId>,
}

/// Body of [`TicketLayouts::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateTicketLayout {
    /// Internal description.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<String>,
    /// Whether this is the default layout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
    /// Dynamic layout specification, following pretix' `pdf-layout.schema.json`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<serde_json::Value>,
    /// Background PDF uploaded with [`Client::upload`](crate::Client::upload).
    /// `Some(None)` restores pretix' default background.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub background: Option<Option<FileId>>,
}

/// Body of [`TicketPdfRenderer::render_batch`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct RenderBatch {
    /// The tickets to render (at most 1000).
    pub parts: Vec<RenderBatchPart>,
}

/// A ticket to render in [`RenderBatch`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct RenderBatchPart {
    /// ID of the order position to render.
    pub orderposition: u64,
    /// Sales channel used to select the layout instead of the order's channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub override_channel: Option<String>,
    /// Layout ID to use instead of the automatically selected one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_layout: Option<u64>,
}

/// Endpoints for the PDF ticket layouts of an event. Created with
/// [`EventScope::ticket_layouts`].
#[derive(Debug, Clone)]
pub struct TicketLayouts {
    client: Client,
    path: Path,
}

/// Endpoint listing the product assignments of all ticket layouts of an event. Created
/// with [`EventScope::ticket_layout_items`].
#[derive(Debug, Clone)]
pub struct TicketLayoutItems {
    client: Client,
    path: Path,
}

/// Endpoint for rendering tickets of an event into one PDF. Created with
/// [`EventScope::ticket_pdf_renderer`].
#[derive(Debug, Clone)]
pub struct TicketPdfRenderer {
    client: Client,
    path: Path,
}

impl EventScope {
    /// PDF ticket layouts of this event.
    #[must_use]
    pub fn ticket_layouts(&self) -> TicketLayouts {
        TicketLayouts {
            client: self.client(),
            path: self.path().join("ticketlayouts"),
        }
    }

    /// Product assignments of the PDF ticket layouts of this event.
    #[must_use]
    pub fn ticket_layout_items(&self) -> TicketLayoutItems {
        TicketLayoutItems {
            client: self.client(),
            path: self.path().join("ticketlayoutitems"),
        }
    }

    /// Batch rendering of PDF tickets of this event.
    #[must_use]
    pub fn ticket_pdf_renderer(&self) -> TicketPdfRenderer {
        TicketPdfRenderer {
            client: self.client(),
            path: self.path().join("ticketpdfrenderer"),
        }
    }
}

impl TicketLayouts {
    /// Lists ticket layouts.
    pub fn list(&self, params: &ListParams) -> Request<Page<TicketLayout>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one ticket layout.
    pub fn get(&self, id: u64) -> Request<TicketLayout> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a ticket layout.
    pub fn create(&self, body: &CreateTicketLayout) -> Request<TicketLayout> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a ticket layout.
    pub fn update(&self, id: u64, body: &UpdateTicketLayout) -> Request<TicketLayout> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a ticket layout.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}

impl TicketLayoutItems {
    /// Lists the assignments of products to ticket layouts.
    pub fn list(&self, params: &ListParams) -> Request<Page<TicketLayoutItem>> {
        self.client.get(&self.path).query(params).page()
    }
}

impl TicketPdfRenderer {
    /// Starts rendering a set of tickets into one combined PDF file (experimental).
    ///
    /// Poll the result with [`ExportJob::download_job`].
    pub fn render_batch(&self, body: &RenderBatch) -> Request<ExportJob> {
        self.client
            .post(&self.path.join("render_batch"))
            .json_body(body)
            .json()
    }
}
