# pretix

[![crates.io](https://img.shields.io/crates/v/pretix?style=flat-square)](https://crates.io/crates/pretix)
[![docs.rs](https://img.shields.io/docsrs/pretix?style=flat-square)](https://docs.rs/pretix)

**Unofficial** async Rust client for the [pretix](https://pretix.eu) REST API.
This project is not affiliated with rami.io GmbH.

## Features

- Covers the documented pretix REST API: organizers, events, products, orders, invoices,
  check-in, vouchers, customers, teams, exports and more (plugin and pretix Hosted
  endpoints behind the `plugins` and `hosted` features).
- Pagination: get the original page, or follow all pages with `.all()`, `.stream()`
  or `.pages()`. Page cursors are serializable for step-by-step processing.
- Typed errors with access to pretix' `detail` and field validation messages, plus
  `is_retryable()` / `retry_after()` classification.
- Automatic, conservative retries that honor `Retry-After` (only for requests that are
  safe to repeat), and support for idempotency keys.
- Conditional fetching (`If-Modified-Since`, `X-Page-Generated`).
- Background jobs (exports, ticket rendering) as single-step polls or awaited to completion.
- Forward compatible: unknown fields are ignored, unknown enum values are preserved.
- All futures and streams are `Send`; all response types round-trip through serde, so
  they can be journaled by durable execution engines such as Restate.

## Usage

```toml
[dependencies]
pretix = "0.3"
```

```rust,no_run
use futures_util::TryStreamExt;
use pretix::Client;
use pretix::resources::events::ListEvents;
use pretix::resources::orders::{ListOrders, MarkPaid, OrderStatus};

#[tokio::main]
async fn main() -> pretix::Result<()> {
    let client = Client::builder()
        .base_url("https://pretix.eu/api/v1/") // or your self-hosted instance
        .token(std::env::var("PRETIX_TOKEN").unwrap())
        .build()?;

    let org = client.organizer("bigevents");

    // One page, as returned by the API.
    let page = org.events().list(&ListEvents::default()).await?;
    println!("{} events", page.count);

    // All events across all pages.
    for event in org.events().list(&ListEvents::default()).all().await? {
        println!("{}: {:?}", event.slug, event.name.localize("en"));
    }

    // Stream pending orders page by page.
    let orders = org.event("sampleconf").orders();
    let params = ListOrders::builder().status(OrderStatus::Pending).build();
    let mut pending = orders.list(&params).stream();
    while let Some(order) = pending.try_next().await? {
        println!("{} {}", order.code, order.total);
    }

    // Mutations can carry an idempotency key, which makes them safe to retry.
    orders
        .mark_paid("ABC12", &MarkPaid::default())
        .idempotency_key("2b1c6f0e-8a52-4d43-9e57-3f1d3c4b7a10")
        .await?;
    Ok(())
}
```

See the [API documentation](https://docs.rs/pretix) for all resources.

### Durable execution

For engines like [Restate](https://restate.dev) that journal side effects and retry on
their own:

- disable client retries with `RetryPolicy::disabled()`,
- journal one page per step using `Page::next_cursor()` and `Client::page(&cursor)`,
- poll jobs once per step (`.await` a `JobStatus`) and sleep durably in between,
- derive idempotency keys deterministically (e.g. from the invocation's random seed),
- map errors with `Error::is_retryable()` to retryable vs. terminal failures.

## Feature flags

| Feature      | Default | Description                                                      |
|--------------|---------|------------------------------------------------------------------|
| `rustls`     | yes     | TLS via rustls                                                   |
| `native-tls` | no      | TLS via the platform implementation                              |
| `plugins`    | no      | Endpoints of plugins not bundled with pretix                     |
| `hosted`     | no      | Endpoints only available on pretix Hosted                        |
| `full`       | no      | `plugins` + `hosted`                                             |

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://opensource.org/licenses/Apache-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
