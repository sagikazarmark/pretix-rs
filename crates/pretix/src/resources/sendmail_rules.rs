//! Scheduled email rules of the bundled sendmail plugin.
//!
//! The `pretix.plugins.sendmail` plugin must be active for the event.
//!
//! <https://docs.pretix.eu/dev/api/resources/sendmail_rules.html>

use jiff::Timestamp;
use jiff::civil::Time;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::types::{LocalizedString, string_enum};
use crate::{Client, EventScope, ListParams, Page, Request};

/// A rule that sends emails automatically at a specific point in time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct SendmailRule {
    /// Internal ID of the rule.
    pub id: u64,
    /// If `false`, the rule is ignored.
    pub enabled: bool,
    /// Email subject.
    pub subject: LocalizedString,
    /// Email body.
    pub template: LocalizedString,
    /// Whether the email is sent to buyers of all products.
    pub all_products: bool,
    /// Product IDs, if not all products.
    #[serde(default)]
    pub limit_products: Vec<u64>,
    /// **Deprecated**: use `restrict_to_status`. Whether pending orders are included.
    #[serde(default)]
    pub include_pending: Option<bool>,
    /// Order states to restrict recipients to.
    #[serde(default)]
    pub restrict_to_status: Vec<SendmailOrderStatus>,
    /// Check-in status to restrict recipients to (`None`: no filtering).
    pub checked_in_status: Option<CheckedInStatus>,
    /// Whether the email is sent at the absolute time `send_date`.
    pub date_is_absolute: bool,
    /// When to send the email, if `date_is_absolute`.
    pub send_date: Option<Timestamp>,
    /// Days before/after the event the email is sent, if not `date_is_absolute`.
    pub send_offset_days: Option<i64>,
    /// Time of day the email is sent, if not `date_is_absolute`.
    pub send_offset_time: Option<Time>,
    /// Whether `send_offset_days` is relative to the event end (otherwise the start).
    pub offset_to_event_end: bool,
    /// Whether `send_offset_days` counts days after (otherwise before) the event.
    pub offset_is_after: bool,
    /// Who receives the email.
    pub send_to: SendTo,
}

string_enum! {
    /// Order states that recipients of a [`SendmailRule`] can be restricted to.
    pub enum SendmailOrderStatus {
        /// Paid.
        Paid = "p",
        /// Expired.
        Expired = "e",
        /// Canceled.
        Canceled = "c",
        /// Pending approval.
        PendingApproval = "n__pending_approval",
        /// Payment pending.
        PaymentPending = "n__not_pending_approval_and_not_valid_if_pending",
        /// Payment pending, but already confirmed.
        ValidIfPending = "n__valid_if_pending",
        /// Pending with payment overdue.
        PendingOverdue = "n__pending_overdue",
    }
}

string_enum! {
    /// Check-in status filter of a [`SendmailRule`].
    pub enum CheckedInStatus {
        /// Attendees that are or have been checked in.
        CheckedIn = "checked_in",
        /// Attendees that have not checked in.
        NoCheckin = "no_checkin",
    }
}

string_enum! {
    /// Recipients of a [`SendmailRule`].
    pub enum SendTo {
        /// One email per order, to the customer.
        Orders = "orders",
        /// One email per attendee.
        Attendees = "attendees",
        /// Both customers and attendees.
        Both = "both",
    }
}

/// Body of [`SendmailRules::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateSendmailRule {
    /// Email subject.
    #[builder(into)]
    pub subject: LocalizedString,
    /// Email body.
    #[builder(into)]
    pub template: LocalizedString,
    /// If `false`, the rule is ignored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Whether the email is sent to buyers of all products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_products: Option<bool>,
    /// Product IDs, if not all products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_products: Option<Vec<u64>>,
    /// **Deprecated**: use `restrict_to_status`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_pending: Option<bool>,
    /// Order states to restrict recipients to (default: paid and confirmed pending).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrict_to_status: Option<Vec<SendmailOrderStatus>>,
    /// Check-in status to restrict recipients to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked_in_status: Option<CheckedInStatus>,
    /// Whether the email is sent at the absolute time `send_date`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_is_absolute: Option<bool>,
    /// When to send the email, if `date_is_absolute`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_date: Option<Timestamp>,
    /// Days before/after the event the email is sent, if not `date_is_absolute`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_offset_days: Option<i64>,
    /// Time of day the email is sent, if not `date_is_absolute`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_offset_time: Option<Time>,
    /// Whether `send_offset_days` is relative to the event end.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_to_event_end: Option<bool>,
    /// Whether `send_offset_days` counts days after the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_is_after: Option<bool>,
    /// Who receives the email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_to: Option<SendTo>,
}

/// Body of [`SendmailRules::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateSendmailRule {
    /// If `false`, the rule is ignored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Email subject.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub subject: Option<LocalizedString>,
    /// Email body.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub template: Option<LocalizedString>,
    /// Whether the email is sent to buyers of all products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_products: Option<bool>,
    /// Product IDs, if not all products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_products: Option<Vec<u64>>,
    /// **Deprecated**: use `restrict_to_status`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_pending: Option<bool>,
    /// Order states to restrict recipients to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrict_to_status: Option<Vec<SendmailOrderStatus>>,
    /// Check-in status to restrict recipients to.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub checked_in_status: Option<Option<CheckedInStatus>>,
    /// Whether the email is sent at the absolute time `send_date`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_is_absolute: Option<bool>,
    /// When to send the email, if `date_is_absolute`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub send_date: Option<Option<Timestamp>>,
    /// Days before/after the event the email is sent, if not `date_is_absolute`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub send_offset_days: Option<Option<i64>>,
    /// Time of day the email is sent, if not `date_is_absolute`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub send_offset_time: Option<Option<Time>>,
    /// Whether `send_offset_days` is relative to the event end.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_to_event_end: Option<bool>,
    /// Whether `send_offset_days` counts days after the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_is_after: Option<bool>,
    /// Who receives the email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_to: Option<SendTo>,
}

/// Endpoints for the scheduled email rules of an event. Created with
/// [`EventScope::sendmail_rules`].
#[derive(Debug, Clone)]
pub struct SendmailRules {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Scheduled email rules of this event.
    #[must_use]
    pub fn sendmail_rules(&self) -> SendmailRules {
        SendmailRules {
            client: self.client(),
            path: self.path().join("sendmail_rules"),
        }
    }
}

impl SendmailRules {
    /// Lists rules.
    pub fn list(&self, params: &ListParams) -> Request<Page<SendmailRule>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one rule.
    pub fn get(&self, id: u64) -> Request<SendmailRule> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a rule.
    pub fn create(&self, body: &CreateSendmailRule) -> Request<SendmailRule> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a rule.
    pub fn update(&self, id: u64, body: &UpdateSendmailRule) -> Request<SendmailRule> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a rule. Rules that are in use cannot be deleted.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
