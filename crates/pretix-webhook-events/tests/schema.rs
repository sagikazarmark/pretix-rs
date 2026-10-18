//! JSON Schemas of webhook event payloads.

#![cfg(feature = "schemars")]

use pretix_webhook_events::{
    CheckinEvent, CustomerEvent, EventEvent, GiftCardEvent, GiftCardTransactionEvent, ItemEvent,
    OrderEvent, SubeventEvent, UnknownEvent, VoucherEvent, WaitingListEvent, WebhookEvent,
};
use schemars::{JsonSchema, schema_for};
use serde_json::{Value, json};

fn validator<T: JsonSchema>() -> jsonschema::Validator {
    let schema = schema_for!(T);
    jsonschema::validator_for(schema.as_value()).expect("generated schema should be valid")
}

#[track_caller]
fn assert_valid<T: JsonSchema>(instance: &Value) {
    if let Err(error) = validator::<T>().validate(instance) {
        panic!(
            "{instance} does not match the {} schema: {error}",
            T::schema_name()
        );
    }
}

/// Validates the serialized event against both the union schema and the
/// schema of the payload type it was dispatched to.
#[track_caller]
fn assert_event_valid(input: &Value) {
    let event: WebhookEvent =
        serde_json::from_value(input.clone()).expect("payload should deserialize");
    let output = serde_json::to_value(&event).expect("payload should serialize");

    assert_valid::<WebhookEvent>(input);
    assert_valid::<WebhookEvent>(&output);

    match event {
        WebhookEvent::Order(_) => assert_valid::<OrderEvent>(&output),
        WebhookEvent::Checkin(_) => assert_valid::<CheckinEvent>(&output),
        WebhookEvent::Event(_) => assert_valid::<EventEvent>(&output),
        WebhookEvent::Voucher(_) => assert_valid::<VoucherEvent>(&output),
        WebhookEvent::Subevent(_) => assert_valid::<SubeventEvent>(&output),
        WebhookEvent::Item(_) => assert_valid::<ItemEvent>(&output),
        WebhookEvent::WaitingList(_) => assert_valid::<WaitingListEvent>(&output),
        WebhookEvent::Customer(_) => assert_valid::<CustomerEvent>(&output),
        WebhookEvent::GiftCard(_) => assert_valid::<GiftCardEvent>(&output),
        WebhookEvent::GiftCardTransaction(_) => assert_valid::<GiftCardTransactionEvent>(&output),
        WebhookEvent::Unknown(_) => assert_valid::<UnknownEvent>(&output),
    }
}

#[test]
fn every_payload_shape_matches_its_schema() {
    for input in [
        json!({"notification_id": 1, "organizer": "acme", "event": "demo", "code": "ABC23", "action": "pretix.event.order.placed"}),
        json!({"notification_id": 2, "organizer": "acme", "event": "demo", "code": "ABC23", "action": "pretix.event.checkin"}),
        json!({
            "notification_id": 3,
            "organizer": "acme",
            "event": "demo",
            "code": "ABC23",
            "action": "pretix.event.checkin",
            "orderposition_id": 91,
            "orderposition_positionid": 2,
            "checkin_list": 3,
            "type": "entry",
            "first_checkin": true
        }),
        json!({"notification_id": 4, "organizer": "acme", "event": "demo", "action": "pretix.event.changed"}),
        json!({"notification_id": 5, "organizer": "acme", "event": "demo", "voucher": 17, "action": "pretix.voucher.changed"}),
        json!({"notification_id": 6, "organizer": "acme", "event": "demo", "subevent": "18", "action": "pretix.subevent.added"}),
        json!({"notification_id": 7, "organizer": "acme", "event": "demo", "item": 19, "action": "pretix.event.item.changed"}),
        json!({"notification_id": 8, "organizer": "acme", "event": "demo", "waitinglistentry": 20, "action": "pretix.event.orders.waitinglist.added"}),
        json!({"notification_id": 9, "organizer": "acme", "customer": "cust-1", "action": "pretix.customer.created"}),
        json!({"notification_id": 10, "issuer_id": 4, "issuer_slug": "acme", "giftcard": 21, "action": "pretix.giftcards.created"}),
        json!({"notification_id": 11, "issuer_id": 4, "issuer_slug": "acme", "giftcard": 21, "action": "pretix.giftcards.transaction.redeemed"}),
        json!({"notification_id": 12, "issuer_id": 4, "issuer_slug": "acme", "acceptor_id": 5, "acceptor_slug": "other", "giftcard": 21, "action": "pretix.giftcards.transaction.redeemed"}),
        json!({"notification_id": 13, "action": "pretix.plugin.badge.printed", "badge_id": 123, "nested": {"value": true}}),
    ] {
        assert_event_valid(&input);
    }
}

#[test]
fn schema_rejects_invalid_common_envelopes() {
    let validator = validator::<WebhookEvent>();

    for input in [
        json!({"notification_id": 1}),
        json!({"notification_id": 1, "action": 42}),
        json!({"notification_id": -1, "action": "pretix.plugin.badge.printed"}),
        json!({"action": "pretix.plugin.badge.printed"}),
        json!("pretix.event.order.placed"),
    ] {
        assert!(
            !validator.is_valid(&input),
            "invalid payload unexpectedly matched the schema: {input}"
        );
    }
}

#[test]
fn payload_schemas_require_their_identity_fields() {
    let order = json!({"notification_id": 1, "organizer": "acme", "event": "demo", "action": "pretix.event.order.placed"});
    assert!(!validator::<OrderEvent>().is_valid(&order));

    let checkin = json!({"notification_id": 1, "organizer": "acme", "event": "demo", "code": "ABC23", "action": "pretix.event.checkin", "type": 3});
    assert!(!validator::<CheckinEvent>().is_valid(&checkin));
}
