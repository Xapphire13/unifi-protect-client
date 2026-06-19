use crate::{RequestError, UnifiProtectClient, models::device_tag::DeviceTag};

impl UnifiProtectClient {
    /// Get all device tags.
    ///
    /// # Examples
    /// ```rust
    /// # use unifi_protect_client::{UnifiProtectClient, models::device_tag::DeviceTag};
    /// # use anyhow::Result;
    /// #
    /// # async fn example() -> Result<()> {
    /// # let client = UnifiProtectClient::new("https://your-unifi-protect-url", "username", "password");
    /// let device_tags = client.get_device_tags().await?;
    /// println!("{:?}", device_tags);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_device_tags(&self) -> Result<Vec<DeviceTag>, RequestError> {
        let device_tags = self
            .make_get_request("proxy/protect/api/device-tags")
            .await?;
        Ok(device_tags)
    }
}
