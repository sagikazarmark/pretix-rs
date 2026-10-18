# pretix-rs

[![ci](https://img.shields.io/github/actions/workflow/status/sagikazarmark/pretix-rs/ci.yaml?style=flat-square&label=ci)](https://github.com/sagikazarmark/pretix-rs/actions/workflows/ci.yaml)
[![openssf scorecard](https://api.securityscorecards.dev/projects/github.com/sagikazarmark/pretix-rs/badge?style=flat-square)](https://securityscorecards.dev/viewer/?uri=github.com/sagikazarmark/pretix-rs)

**Unofficial Rust crates for [pretix](https://pretix.eu): a REST API client and
webhook receivers.**

This project is not affiliated with rami.io GmbH.

## Crates

| Crate | Description | |
|-------|-------------|-|
| [`pretix`](crates/pretix) | Async client for the pretix REST API | [![crates.io](https://img.shields.io/crates/v/pretix?style=flat-square)](https://crates.io/crates/pretix) [![docs.rs](https://img.shields.io/docsrs/pretix?style=flat-square)](https://docs.rs/pretix) |
| [`pretix-webhook`](crates/pretix-webhook) | Framework-independent Tower service for receiving webhooks | [![crates.io](https://img.shields.io/crates/v/pretix-webhook?style=flat-square)](https://crates.io/crates/pretix-webhook) [![docs.rs](https://img.shields.io/docsrs/pretix-webhook?style=flat-square)](https://docs.rs/pretix-webhook) |
| [`pretix-webhook-events`](crates/pretix-webhook-events) | Typed webhook payloads with no HTTP dependency | [![crates.io](https://img.shields.io/crates/v/pretix-webhook-events?style=flat-square)](https://crates.io/crates/pretix-webhook-events) [![docs.rs](https://img.shields.io/docsrs/pretix-webhook-events?style=flat-square)](https://docs.rs/pretix-webhook-events) |
| [`pretix-webhook-cli`](crates/pretix-webhook-cli) | Ready-to-run webhook receiver server | |

All crates are released together and share a version number.

## API client

```toml
[dependencies]
pretix = "0.3"
```

```rust,no_run
use pretix::Client;
use pretix::resources::events::ListEvents;

#[tokio::main]
async fn main() -> pretix::Result<()> {
    let client = Client::builder()
        .base_url("https://pretix.eu/api/v1/") // or your self-hosted instance
        .token(std::env::var("PRETIX_TOKEN").unwrap())
        .build()?;

    let events = client.organizer("bigevents").events();
    for event in events.list(&ListEvents::default()).all().await? {
        println!("{}", event.slug);
    }
    Ok(())
}
```

Pagination, retries, idempotency keys, background jobs, and feature flags are
covered in the [`pretix` README](crates/pretix).

## Webhooks

```toml
[dependencies]
pretix-webhook = "0.3"
pretix-webhook-events = "0.3"
axum = "0.8"
tokio = { version = "1", features = ["macros", "net", "rt-multi-thread"] }
```

```rust,no_run
use std::convert::Infallible;

use axum::{Router, routing::post_service};
use pretix_webhook::WebhookServiceBuilder;
use pretix_webhook_events::WebhookEvent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handler = |event: WebhookEvent| async move {
        println!("{}: {}", event.notification_id(), event.action());
        Ok::<_, Infallible>(())
    };
    let webhook = WebhookServiceBuilder::new()
        .allow_organizer("acmecorp")?
        .allow_event("democon")?
        .build(handler);
    let app = Router::<()>::new().route("/webhook", post_service(webhook));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

Authentication, filtering, and the HTTP response contract are covered in the
[`pretix-webhook` README](crates/pretix-webhook). To receive webhooks without
writing code, use the [CLI](crates/pretix-webhook-cli).

## Development

The workspace requires Rust 1.88 or newer. The [devenv](https://devenv.sh)
shell provides the toolchain and tools; the [justfile](justfile) holds the
recipes CI runs:

```console
devenv shell
just ci
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
