use std::time::Duration;

use jiff::Timestamp;
use pretix::query::{FieldSelection, Ordering};
use pretix::resources::orders::{
    AddPosition, BlockedSecret, CancelPosition, ChangeOrder, CheckinType, ConfirmPayment,
    CreateAnswer, CreateInvoiceAddress, CreateOrder, CreateOrderPayment, CreateOrderPosition,
    CreateOrderRefund, CreatePrintLog, DenyOrder, ExtendOrder, FeeType, GetOrder, GetOrderPosition,
    ListBlockedSecrets, ListOrderPositions, ListOrders, ListRevokedSecrets, MarkCanceled, MarkPaid,
    NewOrderFee, NewOrderPosition, Order, OrderFee, OrderOrdering, OrderPayment, OrderPosition,
    OrderRefund, OrderStatus, PatchPosition, PaymentState, PositionChanges, PrintLog, PrintType,
    ProcessRefund, QuotaCheck, RecalculateTaxes, RefundPayment, RefundSource, RefundState,
    RevokedSecret, RevokedSecretOrdering, UpdateOrder, UpdateOrderPosition,
};
use pretix::{JobStatus, ListParams, PollOptions};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

const EVENT: &str = "/api/v1/organizers/bigevents/events/sampleconf";

#[test]
fn order_models() {
    let order = assert_model::<Order>(&fixture("orders/order"), &[]);
    assert_eq!(order.status, OrderStatus::Paid);
    assert_eq!(order.positions[0].checkins[0].r#type, CheckinType::Entry);
    assert_eq!(order.positions[0].answers[0].option_identifiers.len(), 0);
    assert_eq!(order.payments[0].state, PaymentState::Confirmed);
    assert_model::<Order>(&fixture("orders/order_in_list"), &[]);
    let fee = assert_model::<OrderFee>(&fixture("orders/fee"), &[]);
    assert_eq!(fee.fee_type, FeeType::Payment);
}

#[test]
fn position_models() {
    assert_model::<OrderPosition>(&fixture("orders/position"), &[]);
    assert_model::<OrderPosition>(&fixture("orders/position_in_list"), &[]);
    let position = assert_model::<OrderPosition>(&fixture("orders/position_with_seat"), &[]);
    assert_eq!(position.seat.unwrap().seat_number, "12");
    assert_eq!(position.event.as_deref(), Some("sampleconf"));
    let log = assert_model::<PrintLog>(&fixture("orders/print_log"), &[]);
    assert_eq!(log.r#type, PrintType::Badge);
}

#[test]
fn payment_and_refund_models() {
    assert_model::<OrderPayment>(&fixture("orders/payment"), &[]);
    let refund = assert_model::<OrderRefund>(&fixture("orders/refund"), &[]);
    assert_eq!(refund.state, RefundState::Done);
    assert_eq!(refund.source, RefundSource::Admin);
    assert_model::<OrderRefund>(&fixture("orders/refund_created"), &[]);
}

#[test]
fn secret_models() {
    assert_model::<RevokedSecret>(&fixture("orders/revoked_secret"), &[]);
    assert_model::<BlockedSecret>(&fixture("orders/blocked_secret"), &[]);
}

#[tokio::test]
async fn list_with_filters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}/orders/")))
        .and(query_param("ordering", "-last_modified"))
        .and(query_param("status", "n"))
        .and(query_param("testmode", "false"))
        .and(query_param("modified_since", "2017-12-01T10:00:00Z"))
        .and(query_param("subevent_after", "2017-12-02T00:00:00Z"))
        .and(query_param("include_canceled_positions", "true"))
        .and(query_param("pdf_data", "true"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("X-Page-Generated", "2017-12-01T10:00:00Z")
                .set_body_json(page_of(vec![fixture("orders/order_in_list")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListOrders::builder()
        .ordering(Ordering::desc(OrderOrdering::LastModified))
        .status(OrderStatus::Pending)
        .testmode(false)
        .modified_since("2017-12-01T10:00:00Z".parse::<Timestamp>().unwrap())
        .subevent_after("2017-12-02T00:00:00Z".parse::<Timestamp>().unwrap())
        .include_canceled_positions(true)
        .pdf_data(true)
        .build();
    let page = client
        .organizer("bigevents")
        .event("sampleconf")
        .orders()
        .list(&params)
        .await
        .unwrap();
    assert_eq!(page.results[0].code, "ABC12");
    assert_eq!(
        page.meta.page_generated,
        Some("2017-12-01T10:00:00Z".parse().unwrap())
    );
}

fn query_values(request: &wiremock::Request, key: &str) -> Vec<String> {
    request
        .url
        .query_pairs()
        .filter(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
        .collect()
}

#[tokio::test]
async fn list_with_field_selection() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}/orders/")))
        .and(query_param("status", "p"))
        .and(|request: &wiremock::Request| {
            query_values(request, "include") == ["code", "positions.secret"]
                && query_values(request, "exclude") == ["positions.downloads"]
        })
        .respond_with(ResponseTemplate::new(200).set_body_json(page_of(vec![
            json!({"code": "ABC12", "positions": [{"secret": "z3fsn8jyufm5kpk768q69gkbyr5f4h6w"}]}),
        ])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/orders/"))
        .and(query_param("exclude", "fees"))
        .and(|request: &wiremock::Request| query_values(request, "include").is_empty())
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![json!({"code": "ABC12"})])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let org = client.organizer("bigevents");
    let fields = FieldSelection::builder()
        .include(vec!["code".into(), "positions.secret".into()])
        .exclude(vec!["positions.downloads".into()])
        .build();
    let page = org
        .event("sampleconf")
        .orders()
        .list_with(
            &ListOrders::builder().status(OrderStatus::Paid).build(),
            &fields,
        )
        .await
        .unwrap();
    assert_eq!(
        page.results[0]["positions"][0]["secret"],
        "z3fsn8jyufm5kpk768q69gkbyr5f4h6w"
    );
    let page = org
        .orders()
        .list_with(
            &ListOrders::default(),
            &FieldSelection::builder()
                .exclude(vec!["fees".into()])
                .build(),
        )
        .await
        .unwrap();
    assert_eq!(page.results[0]["code"], "ABC12");
}

#[tokio::test]
async fn organizer_wide_lists() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/orders/"))
        .and(query_param("email", "tester@example.org"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("orders/order")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/organizers/bigevents/orderpositions/"))
        .and(query_param("has_checkin", "false"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("orders/position_with_seat")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let org = client.organizer("bigevents");
    let orders = org
        .orders()
        .list(&ListOrders::builder().email("tester@example.org").build())
        .all()
        .await
        .unwrap();
    assert_eq!(orders[0].event, "sampleconf");
    let positions = org
        .order_positions()
        .list(&ListOrderPositions::builder().has_checkin(false).build())
        .await
        .unwrap();
    assert_eq!(positions.results[0].event.as_deref(), Some("sampleconf"));
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "walks through every endpoint of the resource against one mock server"
)]
async fn get_create_update_delete() {
    let (server, client) = setup().await;
    let order = fixture("orders/order");
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}/orders/ABC12/")))
        .and(query_param("include_canceled_fees", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&order))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}/orders/ABC12/")))
        .and(query_param_is_missing("include_canceled_fees"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&order))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{EVENT}/orders/")))
        .and(body_json(json!({
            "email": "dummy@example.org",
            "locale": "en",
            "sales_channel": "web",
            "api_meta": {"external_id": 42},
            "fees": [{
                "fee_type": "payment",
                "value": "0.25",
                "description": "",
                "internal_type": "",
                "tax_rule": 2
            }],
            "payment_provider": "banktransfer",
            "invoice_address": {
                "is_business": false,
                "company": "Sample company",
                "name_parts": {"full_name": "John Doe"},
                "street": "Sesam Street 12",
                "zipcode": "12345",
                "city": "Sample City",
                "country": "GB",
                "state": "",
                "internal_reference": "",
                "vat_id": ""
            },
            "positions": [{
                "positionid": 1,
                "item": 1,
                "price": "23.00",
                "attendee_name_parts": {"full_name": "Peter"},
                "answers": [{"question": 1, "answer": "23"}],
                "valid_from": null
            }],
            "send_email": null
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&order))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{EVENT}/orders/ABC12/")))
        .and(body_json(json!({
            "email": "other@example.org",
            "locale": "de",
            "comment": "Foo",
            "checkin_attention": true,
            "invoice_address": null
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(&order))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{EVENT}/orders/ABC12/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let orders = client.organizer("bigevents").event("sampleconf").orders();
    orders
        .get_with(
            "ABC12",
            &GetOrder::builder().include_canceled_fees(true).build(),
        )
        .await
        .unwrap();
    orders.get("ABC12").await.unwrap();

    let name = |n: &str| [("full_name".to_owned(), n.to_owned())].into();
    let body = CreateOrder::builder()
        .email("dummy@example.org")
        .locale("en")
        .sales_channel("web")
        .api_meta(
            [("external_id".to_owned(), json!(42))]
                .into_iter()
                .collect(),
        )
        .fees(vec![
            NewOrderFee::builder()
                .fee_type(FeeType::Payment)
                .value("0.25".parse().unwrap())
                .description("")
                .internal_type("")
                .tax_rule(2)
                .build(),
        ])
        .payment_provider("banktransfer")
        .invoice_address(
            CreateInvoiceAddress::builder()
                .is_business(false)
                .company("Sample company")
                .name_parts(name("John Doe"))
                .street("Sesam Street 12")
                .zipcode("12345")
                .city("Sample City")
                .country("GB")
                .state("")
                .internal_reference("")
                .vat_id("")
                .build(),
        )
        .positions(vec![
            NewOrderPosition::builder()
                .positionid(1)
                .item(1)
                .price("23.00".parse().unwrap())
                .attendee_name_parts(name("Peter"))
                .answers(vec![
                    CreateAnswer::builder().question(1).answer("23").build(),
                ])
                .valid_from(None)
                .build(),
        ])
        .send_email(None)
        .build();
    let created = orders.create(&body).await.unwrap();
    assert_eq!(created.code, "ABC12");

    let update = UpdateOrder::builder()
        .email(Some("other@example.org".into()))
        .locale("de")
        .comment("Foo")
        .checkin_attention(true)
        .invoice_address(None)
        .build();
    orders.update("ABC12", &update).await.unwrap();
    orders.delete("ABC12").await.unwrap();
}

async fn mount_action(server: &MockServer, action: &str, body: Option<serde_json::Value>) {
    let mut mock = Mock::given(method("POST")).and(path(format!("{EVENT}/orders/ABC12/{action}/")));
    if let Some(body) = body {
        mock = mock.and(body_json(body));
    }
    mock.respond_with(ResponseTemplate::new(200).set_body_json(fixture("orders/order")))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn state_actions() {
    let (server, client) = setup().await;
    mount_action(&server, "mark_paid", Some(json!({"send_email": true}))).await;
    mount_action(
        &server,
        "mark_canceled",
        Some(json!({"send_email": true, "comment": "Event was canceled.", "cancellation_fee": "5.00"})),
    )
    .await;
    mount_action(&server, "reactivate", None).await;
    mount_action(&server, "mark_pending", None).await;
    mount_action(&server, "mark_expired", None).await;
    mount_action(
        &server,
        "extend",
        Some(json!({"expires": "2017-10-28", "force": false})),
    )
    .await;
    mount_action(&server, "approve", None).await;
    mount_action(
        &server,
        "deny",
        Some(json!({"send_email": true, "comment": "You're not a business customer!"})),
    )
    .await;
    mount_action(&server, "regenerate_secrets", None).await;
    Mock::given(method("POST"))
        .and(path(format!("{EVENT}/orders/ABC12/create_invoice/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("invoices/invoice")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{EVENT}/orders/ABC12/resend_link/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let orders = client.organizer("bigevents").event("sampleconf").orders();
    orders
        .mark_paid("ABC12", &MarkPaid::builder().send_email(true).build())
        .await
        .unwrap();
    let cancel = MarkCanceled::builder()
        .send_email(true)
        .comment("Event was canceled.")
        .cancellation_fee("5.00".parse().unwrap())
        .build();
    orders.mark_canceled("ABC12", &cancel).await.unwrap();
    orders.reactivate("ABC12").await.unwrap();
    orders.mark_pending("ABC12").await.unwrap();
    orders.mark_expired("ABC12").await.unwrap();
    let extend = ExtendOrder::builder()
        .expires(jiff::civil::date(2017, 10, 28))
        .force(false)
        .build();
    orders.extend("ABC12", &extend).await.unwrap();
    orders.approve("ABC12").await.unwrap();
    let deny = DenyOrder::builder()
        .send_email(true)
        .comment("You're not a business customer!")
        .build();
    orders.deny("ABC12", &deny).await.unwrap();
    orders.regenerate_secrets("ABC12").await.unwrap();
    let invoice = orders.create_invoice("ABC12").await.unwrap();
    assert_eq!(invoice.number, "SAMPLECONF-00001");
    orders.resend_link("ABC12").await.unwrap();
}

#[tokio::test]
async fn ticket_downloads() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}/orders/ABC12/download/pdf/")))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({"detail": "pending"})))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}/orders/ABC12/download/pdf/")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "application/pdf")
                .insert_header("Content-Disposition", "attachment; filename=\"ABC12.pdf\"")
                .set_body_bytes("%PDF"),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}/orderpositions/23442/download/pdf/")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "application/pdf")
                .set_body_bytes("%PDF"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let event = client.organizer("bigevents").event("sampleconf");
    let status = event.orders().download("ABC12", "pdf").await.unwrap();
    assert!(status.is_pending());
    let file = event
        .orders()
        .download("ABC12", "pdf")
        .wait(PollOptions::default().with_interval(Duration::from_millis(1)))
        .await
        .unwrap();
    assert_eq!(file.filename.as_deref(), Some("ABC12.pdf"));

    let status = event
        .order_positions()
        .download(23442, "pdf")
        .await
        .unwrap();
    match status {
        JobStatus::Ready(file) => assert_eq!(&file.data[..], b"%PDF"),
        other => panic!("unexpected {other:?}"),
    }
}

#[tokio::test]
async fn change_order() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path(format!("{EVENT}/orders/ABC12/change/")))
        .and(query_param("check_quotas", "false"))
        .and(body_json(json!({
            "patch_positions": [{
                "position": 12374,
                "body": {"item": 12, "variation": null, "subevent": 562, "seat": "seat-guid-2", "price": "99.99", "tax_rule": 15}
            }],
            "cancel_positions": [{"position": 12373}],
            "create_positions": [{"item": 12, "addon_to": 1, "attendee_name": "Peter"}],
            "create_fees": [{"fee_type": "other", "value": "1.50", "description": "Example Fee"}],
            "reissue_invoice": true,
            "send_email": true,
            "recalculate_taxes": "keep_gross"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("orders/order")))
        .expect(1)
        .mount(&server)
        .await;

    let body = ChangeOrder::builder()
        .patch_positions(vec![
            PatchPosition::builder()
                .position(12374)
                .body(
                    PositionChanges::builder()
                        .item(12)
                        .variation(None)
                        .subevent(562)
                        .seat(Some("seat-guid-2".into()))
                        .price("99.99".parse().unwrap())
                        .tax_rule(15)
                        .build(),
                )
                .build(),
        ])
        .cancel_positions(vec![CancelPosition::builder().position(12373).build()])
        .create_positions(vec![
            AddPosition::builder()
                .item(12)
                .addon_to(1)
                .attendee_name("Peter")
                .build(),
        ])
        .create_fees(vec![
            NewOrderFee::builder()
                .fee_type(FeeType::Other)
                .value("1.50".parse().unwrap())
                .description("Example Fee")
                .build(),
        ])
        .reissue_invoice(true)
        .send_email(true)
        .recalculate_taxes(RecalculateTaxes::KeepGross)
        .build();
    client
        .organizer("bigevents")
        .event("sampleconf")
        .orders()
        .change_with("ABC12", &body, &QuotaCheck::disabled())
        .await
        .unwrap();
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "walks through every endpoint of the resource against one mock server"
)]
async fn order_positions() {
    let (server, client) = setup().await;
    let position = fixture("orders/position");
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}/orderpositions/")))
        .and(query_param("item__in", "1,2"))
        .and(query_param("order__status__in", "n,p"))
        .and(query_param("voucher", "17"))
        .and(query_param("voucher__code", "ABC"))
        .and(query_param("ordering", "order__datetime"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("orders/position_in_list")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}/orderpositions/23442/")))
        .and(query_param("pdf_data", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&position))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{EVENT}/orderpositions/")))
        .and(query_param("check_quotas", "false"))
        .and(body_json(
            json!({"order": "ABC12", "item": 5, "addon_to": 1}),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(&position))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{EVENT}/orderpositions/23442/")))
        .and(query_param_is_missing("check_quotas"))
        .and(body_json(
            json!({"attendee_email": "other@example.org", "company": null}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&position))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{EVENT}/orderpositions/23442/")))
        .and(query_param("check_quotas", "false"))
        .and(body_json(
            json!({"attendee_name": "Peter", "item": 6, "variation": null}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&position))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{EVENT}/orderpositions/23442/")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    for action in ["add_block", "remove_block"] {
        Mock::given(method("POST"))
            .and(path(format!("{EVENT}/orderpositions/23442/{action}/")))
            .and(body_json(json!({"name": "api:block1"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(&position))
            .expect(1)
            .mount(&server)
            .await;
    }
    Mock::given(method("POST"))
        .and(path(format!(
            "{EVENT}/orderpositions/23442/regenerate_secrets/"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(&position))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{EVENT}/orderpositions/23442/printlog/")))
        .and(body_json(json!({
            "datetime": "2024-09-19T11:37:00Z",
            "source": "pretixPOS",
            "type": "badge",
            "successful": true,
            "info": {"cashier": 1234}
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("orders/print_log")))
        .expect(1)
        .mount(&server)
        .await;

    let positions = client
        .organizer("bigevents")
        .event("sampleconf")
        .order_positions();
    let params = ListOrderPositions::builder()
        .item_in(vec![1, 2])
        .order_status_in(vec![OrderStatus::Pending, OrderStatus::Paid])
        .voucher(17)
        .voucher_code("ABC")
        .ordering(pretix::resources::orders::OrderPositionOrdering::OrderDatetime)
        .build();
    let page = positions.list(&params).await.unwrap();
    assert_eq!(page.results[0].id, 23442);
    positions
        .get_with(23442, &GetOrderPosition::builder().pdf_data(true).build())
        .await
        .unwrap();
    let create = CreateOrderPosition::builder()
        .order("ABC12")
        .item(5)
        .addon_to(1)
        .build();
    positions
        .create_with(&create, &QuotaCheck::disabled())
        .await
        .unwrap();
    let update = UpdateOrderPosition::builder()
        .attendee_email(Some("other@example.org".into()))
        .company(None)
        .build();
    positions.update(23442, &update).await.unwrap();
    let update = UpdateOrderPosition::builder()
        .attendee_name("Peter")
        .item(6)
        .variation(None)
        .build();
    positions
        .update_with(23442, &update, &QuotaCheck::disabled())
        .await
        .unwrap();
    positions.delete(23442).await.unwrap();
    positions.add_block(23442, "api:block1").await.unwrap();
    positions.remove_block(23442, "api:block1").await.unwrap();
    positions.regenerate_secrets(23442).await.unwrap();
    let mut info = serde_json::Map::new();
    info.insert("cashier".into(), json!(1234));
    let log = CreatePrintLog::builder()
        .datetime("2024-09-19T13:37:00+02:00".parse().unwrap())
        .source("pretixPOS")
        .r#type(PrintType::Badge)
        .successful(true)
        .info(info)
        .build();
    let log = positions.printlog(23442, &log).await.unwrap();
    assert_eq!(log.id, 1234);
}

#[tokio::test]
async fn payments() {
    let (server, client) = setup().await;
    let base = format!("{EVENT}/orders/ABC12/payments");
    let payment = fixture("orders/payment");
    Mock::given(method("GET"))
        .and(path(format!("{base}/")))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_of(vec![payment.clone()])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}/1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&payment))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{base}/")))
        .and(body_json(json!({
            "state": "confirmed",
            "amount": "23.00",
            "payment_date": "2017-12-04T12:13:12Z",
            "info": {},
            "send_email": false,
            "provider": "banktransfer"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&payment))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{base}/1/confirm/")))
        .and(body_json(json!({"send_email": true, "force": false})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&payment))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{base}/1/cancel/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&payment))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{base}/1/refund/")))
        .and(body_json(
            json!({"amount": "23.00", "comment": "Overpayment", "mark_canceled": false}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("orders/refund")))
        .expect(1)
        .mount(&server)
        .await;

    let payments = client
        .organizer("bigevents")
        .event("sampleconf")
        .orders()
        .payments("ABC12");
    let page = payments
        .list(&ListParams::builder().page(2).build())
        .await
        .unwrap();
    assert_eq!(page.results[0].local_id, 1);
    payments.get(1).await.unwrap();
    let create = CreateOrderPayment::builder()
        .state(PaymentState::Confirmed)
        .amount("23.00".parse().unwrap())
        .payment_date("2017-12-04T12:13:12Z".parse().unwrap())
        .info(json!({}))
        .send_email(false)
        .provider("banktransfer")
        .build();
    payments.create(&create).await.unwrap();
    let confirm = ConfirmPayment::builder()
        .send_email(true)
        .force(false)
        .build();
    payments.confirm(1, &confirm).await.unwrap();
    payments.cancel(1).await.unwrap();
    let refund = RefundPayment::builder()
        .amount("23.00".parse().unwrap())
        .comment("Overpayment")
        .mark_canceled(false)
        .build();
    let refund = payments.refund(1, &refund).await.unwrap();
    assert_eq!(refund.source, RefundSource::Admin);
}

#[tokio::test]
async fn refunds() {
    let (server, client) = setup().await;
    let base = format!("{EVENT}/orders/ABC12/refunds");
    let refund = fixture("orders/refund");
    Mock::given(method("GET"))
        .and(path(format!("{base}/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_of(vec![refund.clone()])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}/1/")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&refund))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{base}/")))
        .and(body_json(json!({
            "state": "created",
            "source": "admin",
            "amount": "23.00",
            "payment": 1,
            "comment": "Cancellation",
            "provider": "manual",
            "info": {"reference": "R-1"},
            "mark_canceled": false,
            "mark_pending": true
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(fixture("orders/refund_created")))
        .expect(1)
        .mount(&server)
        .await;
    for action in ["done", "cancel"] {
        Mock::given(method("POST"))
            .and(path(format!("{base}/1/{action}/")))
            .respond_with(ResponseTemplate::new(200).set_body_json(&refund))
            .expect(1)
            .mount(&server)
            .await;
    }
    Mock::given(method("POST"))
        .and(path(format!("{base}/1/process/")))
        .and(body_json(json!({"mark_canceled": false})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&refund))
        .expect(1)
        .mount(&server)
        .await;

    let refunds = client
        .organizer("bigevents")
        .event("sampleconf")
        .orders()
        .refunds("ABC12");
    refunds.list(&ListParams::default()).await.unwrap();
    refunds.get(1).await.unwrap();
    let create = CreateOrderRefund::builder()
        .state(RefundState::Created)
        .source(RefundSource::Admin)
        .amount("23.00".parse().unwrap())
        .payment(1)
        .comment("Cancellation")
        .provider("manual")
        .info(json!({"reference": "R-1"}))
        .mark_canceled(false)
        .mark_pending(true)
        .build();
    let created = refunds.create(&create).await.unwrap();
    assert_eq!(created.execution_date, None);
    refunds.done(1).await.unwrap();
    refunds
        .process(1, &ProcessRefund::builder().mark_canceled(false).build())
        .await
        .unwrap();
    refunds.cancel(1).await.unwrap();
}

#[tokio::test]
async fn revoked_and_blocked_secrets() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}/revokedsecrets/")))
        .and(query_param("ordering", "-created"))
        .and(query_param("created_since", "2017-12-01T10:00:00Z"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("orders/revoked_secret")])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{EVENT}/blockedsecrets/")))
        .and(query_param("blocked", "true"))
        .and(query_param("updated_since", "2017-12-01T10:00:00Z"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(page_of(vec![fixture("orders/blocked_secret")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let event = client.organizer("bigevents").event("sampleconf");
    let since: Timestamp = "2017-12-01T10:00:00Z".parse().unwrap();
    let revoked = event
        .revoked_secrets()
        .list(
            &ListRevokedSecrets::builder()
                .ordering(Ordering::desc(RevokedSecretOrdering::Created))
                .created_since(since)
                .build(),
        )
        .await
        .unwrap();
    assert_eq!(revoked.results[0].secret, "k24fiuwvu8kxz3y1");
    let blocked = event
        .blocked_secrets()
        .list(
            &ListBlockedSecrets::builder()
                .blocked(true)
                .updated_since(since)
                .build(),
        )
        .await
        .unwrap();
    assert!(blocked.results[0].blocked);
}
