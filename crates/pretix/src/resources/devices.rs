//! Devices: organizer-level management of devices such as scanners and POS terminals.
//!
//! <https://docs.pretix.eu/dev/api/resources/devices.html>

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::{Client, ListParams, OrganizerScope, Page, Request};

/// A device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct Device {
    /// Internal ID of the device within this organizer.
    pub device_id: u64,
    /// Unique identifier of this device.
    pub unique_serial: String,
    /// Device name.
    pub name: String,
    /// Whether this device has access to all events.
    pub all_events: bool,
    /// Slugs of the events this device has access to.
    #[serde(default)]
    pub limit_events: Vec<String>,
    /// Hardware manufacturer.
    pub hardware_brand: Option<String>,
    /// Hardware model.
    pub hardware_model: Option<String>,
    /// Operating system name.
    pub os_name: Option<String>,
    /// Operating system version.
    pub os_version: Option<String>,
    /// Software product.
    pub software_brand: Option<String>,
    /// Software version.
    pub software_version: Option<String>,
    /// Creation time.
    pub created: Timestamp,
    /// Time of initialization.
    pub initialized: Option<Timestamp>,
    /// Token for initialization (only visible with write permission).
    #[serde(default)]
    pub initialization_token: Option<String>,
    /// Whether this device no longer has access.
    pub revoked: bool,
    /// Name of the security profile restricting API access.
    pub security_profile: String,
}

/// Body of [`Devices::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateDevice {
    /// Device name.
    #[builder(into)]
    pub name: String,
    /// Whether this device has access to all events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_events: Option<bool>,
    /// Slugs of the events this device has access to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_events: Option<Vec<String>>,
    /// Name of the security profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub security_profile: Option<String>,
}

/// Body of [`Devices::update`]. Unset fields are left unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateDevice {
    /// Device name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<String>,
    /// Whether this device has access to all events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_events: Option<bool>,
    /// Slugs of the events this device has access to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_events: Option<Vec<String>>,
    /// Name of the security profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub security_profile: Option<String>,
}

/// Endpoints for the devices of an organizer. Created with [`OrganizerScope::devices`].
#[derive(Debug, Clone)]
pub struct Devices {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Devices of this organizer.
    #[must_use]
    pub fn devices(&self) -> Devices {
        Devices {
            client: self.client(),
            path: self.path().join("devices"),
        }
    }
}

impl Devices {
    /// Lists devices.
    pub fn list(&self, params: &ListParams) -> Request<Page<Device>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one device.
    pub fn get(&self, device_id: u64) -> Request<Device> {
        self.client.get(&self.path.join(device_id)).json()
    }

    /// Creates a device. Use the returned `initialization_token` to set it up.
    pub fn create(&self, body: &CreateDevice) -> Request<Device> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a device.
    pub fn update(&self, device_id: u64, body: &UpdateDevice) -> Request<Device> {
        self.client
            .patch(&self.path.join(device_id))
            .json_body(body)
            .json()
    }
}
