//! Scheduled exports: data exports sent by email at specific times.
//!
//! Scheduled exports exist on event and organizer level. They can only be created with
//! OAuth authentication (the authenticated user becomes the owner), but can be changed
//! with token authentication afterwards.
//!
//! <https://docs.pretix.eu/dev/api/resources/scheduled_exports.html>

use jiff::Timestamp;
use jiff::civil::Time;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Ordering;
use crate::types::string_enum;
use crate::{Client, EventScope, OrganizerScope, Page, Request};

/// A scheduled export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ScheduledExport {
    /// Internal ID of the schedule.
    pub id: u64,
    /// Email address of the user who created the schedule (experimental). The owner
    /// always receives the export, which only contains data the owner may access.
    pub owner: String,
    /// Identifier of the exporter to run, see
    /// [`Exporter`](crate::resources::exporters::Exporter).
    pub export_identifier: String,
    /// Input data for the exporter.
    #[serde(default)]
    pub export_form_data: serde_json::Map<String, serde_json::Value>,
    /// Language to run the export in.
    pub locale: String,
    /// Comma-separated additional recipients (or empty).
    pub mail_additional_recipients: String,
    /// Comma-separated additional recipients in copy (or empty).
    pub mail_additional_recipients_cc: String,
    /// Comma-separated additional recipients in blind copy (or empty).
    pub mail_additional_recipients_bcc: String,
    /// Email subject.
    pub mail_subject: String,
    /// Email text.
    pub mail_template: String,
    /// `RRULE` (RFC 5545) determining the **days** the export runs on.
    pub schedule_rrule: Option<String>,
    /// Time of day to run the export on the scheduled days.
    pub schedule_rrule_time: Time,
    /// Time zone the schedule is interpreted in. Only for organizer-level exports;
    /// event-level exports use the event's time zone.
    #[serde(default)]
    pub timezone: Option<String>,
    /// Next planned execution (computed by the server).
    pub schedule_next_run: Option<Timestamp>,
    /// Number of consecutive failures. After 5 failures, the schedule no longer runs.
    /// Changing the schedule resets the counter.
    pub error_counter: u32,
}

string_enum! {
    /// Fields scheduled exports can be sorted by.
    pub enum ScheduledExportOrdering {
        /// By ID (default).
        Id = "id",
        /// By exporter identifier.
        ExportIdentifier = "export_identifier",
        /// By next planned execution.
        ScheduleNextRun = "schedule_next_run",
    }
}

/// Query parameters for [`ScheduledExports::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListScheduledExports {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<ScheduledExportOrdering>>,
}

/// Body of [`ScheduledExports::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateScheduledExport {
    /// Identifier of the exporter to run.
    #[builder(into)]
    pub export_identifier: String,
    /// Input data for the exporter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_form_data: Option<serde_json::Map<String, serde_json::Value>>,
    /// Language to run the export in (defaults to `en`).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub locale: Option<String>,
    /// Comma-separated additional recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub mail_additional_recipients: Option<String>,
    /// Comma-separated additional recipients in copy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub mail_additional_recipients_cc: Option<String>,
    /// Comma-separated additional recipients in blind copy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub mail_additional_recipients_bcc: Option<String>,
    /// Email subject.
    #[builder(into)]
    pub mail_subject: String,
    /// Email text.
    #[builder(into)]
    pub mail_template: String,
    /// `RRULE` (RFC 5545) determining the days the export runs on.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub schedule_rrule: Option<String>,
    /// Time of day to run the export on the scheduled days.
    pub schedule_rrule_time: Time,
    /// Time zone to interpret the schedule in (organizer-level exports only).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub timezone: Option<String>,
}

/// Body of [`ScheduledExports::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateScheduledExport {
    /// Identifier of the exporter to run.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub export_identifier: Option<String>,
    /// Input data for the exporter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_form_data: Option<serde_json::Map<String, serde_json::Value>>,
    /// Language to run the export in.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub locale: Option<String>,
    /// Comma-separated additional recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub mail_additional_recipients: Option<String>,
    /// Comma-separated additional recipients in copy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub mail_additional_recipients_cc: Option<String>,
    /// Comma-separated additional recipients in blind copy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub mail_additional_recipients_bcc: Option<String>,
    /// Email subject.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub mail_subject: Option<String>,
    /// Email text.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub mail_template: Option<String>,
    /// `RRULE` (RFC 5545) determining the days the export runs on.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub schedule_rrule: Option<Option<String>>,
    /// Time of day to run the export on the scheduled days.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_rrule_time: Option<Time>,
    /// Time zone to interpret the schedule in (organizer-level exports only).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub timezone: Option<String>,
}

/// Endpoints for the scheduled exports of an event or organizer. Created with
/// [`EventScope::scheduled_exports`] or [`OrganizerScope::scheduled_exports`].
#[derive(Debug, Clone)]
pub struct ScheduledExports {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Scheduled exports of this event.
    #[must_use]
    pub fn scheduled_exports(&self) -> ScheduledExports {
        ScheduledExports {
            client: self.client(),
            path: self.path().join("scheduled_exports"),
        }
    }
}

impl OrganizerScope {
    /// Organizer-level scheduled exports.
    #[must_use]
    pub fn scheduled_exports(&self) -> ScheduledExports {
        ScheduledExports {
            client: self.client(),
            path: self.path().join("scheduled_exports"),
        }
    }
}

impl ScheduledExports {
    /// Lists the scheduled exports the credentials have access to.
    pub fn list(&self, params: &ListScheduledExports) -> Request<Page<ScheduledExport>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one scheduled export.
    pub fn get(&self, id: u64) -> Request<ScheduledExport> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Schedules an export. Requires OAuth authentication.
    pub fn create(&self, body: &CreateScheduledExport) -> Request<ScheduledExport> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a scheduled export.
    pub fn update(&self, id: u64, body: &UpdateScheduledExport) -> Request<ScheduledExport> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a scheduled export.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
