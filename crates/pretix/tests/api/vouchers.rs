use pretix::query::Ordering;
use pretix::resources::vouchers::{
    CreateVoucher, ListVouchers, PriceMode, UpdateVoucher, Voucher, VoucherOrdering,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::support::{assert_model, fixture, page_of, setup};

#[test]
fn voucher_model() {
    let voucher = assert_model::<Voucher>(&fixture("vouchers/voucher"), &[]);
    assert_eq!(voucher.price_mode, PriceMode::Set);
    assert_eq!(voucher.min_usages, None);

    let mut with_min_usages = fixture("vouchers/voucher");
    with_min_usages["min_usages"] = json!(2);
    with_min_usages["budget"] = json!("50.00");
    let voucher = assert_model::<Voucher>(&with_min_usages, &[]);
    assert_eq!(voucher.min_usages, Some(2));
}

#[tokio::test]
async fn list_with_filters() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/organizers/bigevents/events/sampleconf/vouchers/",
        ))
        .and(query_param("ordering", "-valid_until"))
        .and(query_param("code", "43K6LKM37FBVR2YG"))
        .and(query_param("search", "test"))
        .and(query_param("max_usages", "1"))
        .and(query_param("redeemed", "0"))
        .and(query_param("active", "true"))
        .and(query_param("block_quota", "false"))
        .and(query_param("allow_ignore_quota", "false"))
        .and(query_param("price_mode", "set"))
        .and(query_param("value", "12.00"))
        .and(query_param("item", "1"))
        .and(query_param("variation", "2"))
        .and(query_param("quota", "3"))
        .and(query_param("tag", "testvoucher"))
        .and(query_param("subevent", "4"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page_of(vec![fixture("vouchers/voucher")])),
        )
        .expect(1)
        .mount(&server)
        .await;

    let params = ListVouchers::builder()
        .ordering(Ordering::desc(VoucherOrdering::ValidUntil))
        .code("43K6LKM37FBVR2YG")
        .search("test")
        .max_usages(1)
        .redeemed(0)
        .active(true)
        .block_quota(false)
        .allow_ignore_quota(false)
        .price_mode(PriceMode::Set)
        .value("12.00".parse().unwrap())
        .item(1)
        .variation(2)
        .quota(3)
        .tag("testvoucher")
        .subevent(4)
        .build();
    let page = client
        .organizer("bigevents")
        .event("sampleconf")
        .vouchers()
        .list(&params)
        .await
        .unwrap();
    assert_eq!(page.results[0].code, "43K6LKM37FBVR2YG");
}

#[tokio::test]
async fn create_and_batch_create() {
    let (server, client) = setup().await;
    let voucher = fixture("vouchers/voucher");
    let mut second = voucher.clone();
    second["id"] = json!(2);
    second["code"] = json!("ASDKLJCYXCASDASD");
    let base = "/api/v1/organizers/bigevents/events/sampleconf/vouchers/";
    Mock::given(method("POST"))
        .and(path(base))
        .and(body_json(json!({
            "code": "43K6LKM37FBVR2YG",
            "max_usages": 1,
            "price_mode": "set",
            "value": "12.00",
            "item": 1,
            "tag": "testvoucher",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(&voucher))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{base}batch_create/")))
        .and(body_json(json!([
            {"code": "43K6LKM37FBVR2YG"},
            {"max_usages": 2},
        ])))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!([voucher, second])))
        .expect(1)
        .mount(&server)
        .await;

    let vouchers = client.organizer("bigevents").event("sampleconf").vouchers();
    let body = CreateVoucher::builder()
        .code("43K6LKM37FBVR2YG")
        .max_usages(1)
        .price_mode(PriceMode::Set)
        .value("12.00".parse().unwrap())
        .item(1)
        .tag("testvoucher")
        .build();
    vouchers.create(&body).await.unwrap();

    let batch = [
        CreateVoucher::builder().code("43K6LKM37FBVR2YG").build(),
        CreateVoucher::builder().max_usages(2).build(),
    ];
    let created = vouchers.batch_create(&batch).await.unwrap();
    assert_eq!(created.len(), 2);
    assert_eq!(created[1].code, "ASDKLJCYXCASDASD");
}

#[tokio::test]
async fn get_update_delete() {
    let (server, client) = setup().await;
    let voucher = fixture("vouchers/voucher");
    let detail = "/api/v1/organizers/bigevents/events/sampleconf/vouchers/1/";
    Mock::given(method("GET"))
        .and(path(detail))
        .respond_with(ResponseTemplate::new(200).set_body_json(&voucher))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(detail))
        .and(body_json(json!({
            "price_mode": "set",
            "value": "24.00",
            "valid_until": null,
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(&voucher))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(detail))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let vouchers = client.organizer("bigevents").event("sampleconf").vouchers();
    vouchers.get(1).await.unwrap();
    let update = UpdateVoucher::builder()
        .price_mode(PriceMode::Set)
        .value(Some("24.00".parse().unwrap()))
        .valid_until(None)
        .build();
    vouchers.update(1, &update).await.unwrap();
    vouchers.delete(1).await.unwrap();
}
