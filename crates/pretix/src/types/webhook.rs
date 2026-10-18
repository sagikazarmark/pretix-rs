use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The body of a webhook call sent by pretix.
///
/// Webhook payloads are not authenticated. Only use them as a trigger to fetch the
/// referenced objects through the API.
///
/// ```json
/// {"notification_id": 123455, "organizer": "acmecorp", "event": "democon", "code": "ABC23", "action": "pretix.event.order.placed"}
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct WebhookNotification {
    /// Unique ID of the notification. pretix may deliver a notification more than once.
    pub notification_id: u64,
    /// Organizer slug.
    pub organizer: String,
    /// Event slug, for event-level notifications.
    #[serde(default)]
    pub event: Option<String>,
    /// Order code, for order related notifications.
    #[serde(default)]
    pub code: Option<String>,
    /// The action type, e.g. `pretix.event.order.placed`.
    pub action: String,
    /// Further, action specific fields (e.g. `orderposition_id`, `checkin_list`).
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::WebhookNotification;

    #[test]
    fn parses_checkin_notification() {
        let notification: WebhookNotification = serde_json::from_str(
            r#"{"notification_id":1,"organizer":"acme","event":"con","code":"ABC23","action":"pretix.event.checkin","orderposition_id":5,"checkin_list":3}"#,
        )
        .unwrap();
        assert_eq!(notification.code.as_deref(), Some("ABC23"));
        assert_eq!(notification.extra["checkin_list"], 3);
    }
}
