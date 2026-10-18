//! The README example: listing events and orders, marking an order as paid.
//!
//! Run with `PRETIX_TOKEN=... cargo run --example basic`.

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
