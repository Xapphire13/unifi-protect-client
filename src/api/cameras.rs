//! Camera management API endpoints.
//!
//! This module provides methods for interacting with UniFi Protect cameras,
//! including listing cameras and updating their settings.

use crate::{
    RequestError, UnifiProtectClient,
    models::camera::{Camera, CameraUpdate},
};

impl UnifiProtectClient {
    /// Retrieves a list of all cameras from the UniFi Protect controller.
    ///
    /// This method fetches all cameras configured in your UniFi Protect system,
    /// including their current settings and status information.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(Vec<Camera>)` - A vector of all cameras on success
    /// - `Err(RequestError)` - An error if the request fails
    ///
    /// # Errors
    ///
    /// This method can return the following errors:
    /// - `RequestError::NetworkError` - Network connectivity issues
    /// - `RequestError::Unauthorized` - Invalid credentials or expired session
    /// - `RequestError::DeserializationError` - Invalid response format
    /// - `RequestError::Unknown` - Other HTTP errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use unifi_protect_client::UnifiProtectClient;
    /// # use anyhow::Result;
    /// #
    /// # async fn example() -> Result<()> {
    /// # let client = UnifiProtectClient::new(
    /// #     "https://192.168.1.1",
    /// #     "admin",
    /// #     "password"
    /// # );
    /// #
    /// match client.list_cameras().await {
    ///     Ok(cameras) => {
    ///         println!("Found {} cameras:", cameras.len());
    ///         for camera in cameras {
    ///             println!("- {} (ID: {})", camera.name, camera.id);
    ///             println!("  Recording mode: {:?}", camera.recording_settings.mode);
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Failed to list cameras: {e}"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_cameras(&self) -> Result<Vec<Camera>, RequestError> {
        let cameras = self.make_get_request("proxy/protect/api/cameras").await?;
        Ok(cameras)
    }

    /// Updates the settings of a specific camera.
    ///
    /// This method allows you to modify camera settings such as recording mode,
    /// privacy settings, and other configurable options.
    ///
    /// # Arguments
    ///
    /// * `camera_id` - The unique identifier of the camera to update
    /// * `updates` - A `CameraUpdate` struct containing the settings to modify
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(())` - Success, camera updated
    /// - `Err(RequestError)` - An error if the update fails
    ///
    /// # Errors
    ///
    /// This method can return the following errors:
    /// - `RequestError::NetworkError` - Network connectivity issues
    /// - `RequestError::Unauthorized` - Invalid credentials or expired session
    /// - `RequestError::Unknown` - Camera not found or other HTTP errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use unifi_protect_client::{UnifiProtectClient, models::camera::*};
    /// # use anyhow::Result;
    /// #
    /// # async fn example() -> Result<()> {
    /// # let client = UnifiProtectClient::new(
    /// #     "https://192.168.1.1",
    /// #     "admin",
    /// #     "password"
    /// # );
    /// #
    /// // Change a camera's recording mode to "always record"
    /// let update = CameraUpdate {
    ///     recording_settings: Some(RecordingSettingsUpdate {
    ///         mode: Some(RecordingMode::Always),
    ///     }),
    /// };
    ///
    /// match client.update_camera("camera-id-123", update).await {
    ///     Ok(()) => println!("Camera updated successfully"),
    ///     Err(e) => eprintln!("Failed to update camera: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_camera(
        &self,
        camera_id: &str,
        updates: CameraUpdate,
    ) -> Result<(), RequestError> {
        let uri = format!("proxy/protect/api/cameras/{camera_id}");
        self.make_patch_request(&uri, updates).await
    }
}
