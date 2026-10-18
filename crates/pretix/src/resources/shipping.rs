//! Shipping of printed tickets: shipping addresses and statuses of orders, and print
//! jobs (plugin `pretix-shipping`).
//!
//! <https://docs.pretix.eu/dev/api/resources/shipping.html>

use serde::{Deserialize, Serialize};

use crate::Download;
use crate::path::Path;
use crate::types::string_enum;
use crate::{Client, EventScope, OrganizerScope, Page, Request};

/// The shipping address of an order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ShippingAddress {
    /// Company name.
    pub company: String,
    /// Name.
    pub name: String,
    /// Street.
    pub street: String,
    /// ZIP code.
    pub zipcode: String,
    /// City.
    pub city: String,
    /// Country code.
    pub country: String,
    /// State (ISO 3166-2 code). Only supported in AU, BR, CA, CN, MY, MX and US.
    pub state: String,
    /// Whether the customer asked not to disclose prices in the shipment.
    pub gift: bool,
}

/// The shipping status of an order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ShippingStatus {
    /// Internal ID of the shipping method.
    pub method: u64,
    /// Shipping state.
    pub status: ShippingState,
    /// Type of the shipping method.
    pub method_type: ShippingMethodType,
}

/// An order whose tickets need to be printed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PrintJob {
    /// Event slug.
    pub event: String,
    /// Order code (documented as `code`, returned as `order`).
    #[serde(alias = "code")]
    pub order: String,
    /// Type of the shipping method.
    pub method: ShippingMethodType,
    /// Shipping state.
    pub status: ShippingState,
}

/// The ticket file of the next print job, returned by [`PrintJobs::poll`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PolledPrintJob {
    /// The order code (`X-Pretix-Order-Code` header). Pass it to [`PrintJobs::ack`]
    /// once printed.
    pub order: Option<String>,
    /// The PDF file.
    pub file: Download,
}

string_enum! {
    /// Shipping state of an order.
    pub enum ShippingState {
        /// Not shipped yet.
        New = "new",
        /// Shipped.
        Shipped = "shipped",
    }
}

string_enum! {
    /// Type of a shipping method.
    pub enum ShippingMethodType {
        /// Shipped by post.
        Ship = "ship",
        /// Delivered online.
        Online = "online",
        /// Collected by the customer.
        Collect = "collect",
    }
}

/// Query parameters for [`PrintJobs::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct ListPrintJobs {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Only jobs with one of these shipping method types.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub method: Vec<ShippingMethodType>,
    /// Only jobs with this shipping state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ShippingState>,
    /// Only jobs of the event with this slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub event: Option<String>,
}

/// Query parameters for [`PrintJobs::poll`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct PollPrintJobs {
    /// Only jobs with one of these shipping method types.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub method: Vec<ShippingMethodType>,
    /// Only jobs with this shipping state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ShippingState>,
    /// Only jobs of the event with this slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub event: Option<String>,
}

/// Endpoints for the shipping information of an event's orders. Created with
/// [`EventScope::shipping`].
#[derive(Debug, Clone)]
pub struct Shipping {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Shipping information of this event's orders.
    #[must_use]
    pub fn shipping(&self) -> Shipping {
        Shipping {
            client: self.client(),
            path: self.path().join("orders"),
        }
    }
}

impl Shipping {
    /// Fetches the shipping address of an order.
    pub fn shipping_address(&self, order_code: &str) -> Request<ShippingAddress> {
        self.client
            .get(&self.path.join(order_code).join("shippingaddress"))
            .json()
    }

    /// Fetches the shipping status of an order.
    pub fn shipping_status(&self, order_code: &str) -> Request<ShippingStatus> {
        self.client
            .get(&self.path.join(order_code).join("shippingstatus"))
            .json()
    }
}

/// Endpoints for the print jobs of an organizer. Created with
/// [`OrganizerScope::print_jobs`].
#[derive(Debug, Clone)]
pub struct PrintJobs {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Print jobs (orders with tickets to print and ship) of this organizer.
    #[must_use]
    pub fn print_jobs(&self) -> PrintJobs {
        PrintJobs {
            client: self.client(),
            path: self.path().join("printjobs"),
        }
    }
}

impl PrintJobs {
    /// Lists print jobs.
    pub fn list(&self, params: &ListPrintJobs) -> Request<Page<PrintJob>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches the ticket PDF of the next job to print.
    pub fn poll(&self, params: &PollPrintJobs) -> Request<PolledPrintJob> {
        self.client
            .get(&self.path.join("poll"))
            .query(params)
            .decode_with(|raw| {
                let order = raw.header_str("x-pretix-order-code").map(str::to_owned);
                Ok(PolledPrintJob {
                    order,
                    file: Download::from_response(raw)?,
                })
            })
    }

    /// Marks an order as shipped.
    pub fn ack(&self, order_code: &str) -> Request<()> {
        self.client
            .post(&self.path.join(order_code).join("ack"))
            .empty()
    }
}
