use serde::{Deserialize, Serialize};

/// Represents a tag associated with a device in UniFi Protect.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTag {
    /// Unique identifier for the device tag.
    pub id: String,

    /// Name of the device tag.
    pub name: String,

    /// List of MAC addresses associated with the device tag.
    pub device_macs: Vec<String>,

    /// Model key associated with the device tag.
    pub model_key: String,
}
