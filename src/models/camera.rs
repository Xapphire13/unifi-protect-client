//! Camera data models and structures.
//!
//! This module defines the data structures used to represent cameras and their
//! settings in the UniFi Protect API.

use serde::{Deserialize, Serialize};
use typesafe_builder::*;

/// Represents a camera in the UniFi Protect system.
///
/// This structure contains the essential information about a camera,
/// including its identification, name, and current recording settings.
///
/// # Examples
///
/// ```rust
/// # use unifi_protect_client::UnifiProtectClient;
/// # use anyhow::Result;
/// # use unifi_protect_client::models::camera::*;
/// #
/// # async fn example() -> Result<()> {
/// # let client = UnifiProtectClient::new(
/// #     "https://192.168.1.1",
/// #     "username",
/// #     "password"
/// # );
/// #
/// let cameras = client.list_cameras().await?;
///
/// for camera in cameras {
///     println!("Camera: {} ({})", camera.name, camera.id);
///     match camera.recording_settings.mode {
///         RecordingMode::Always => println!("Recording continuously"),
///         RecordingMode::Schedule => println!("Recording on schedule"),
///         RecordingMode::Never => println!("Recording disabled"),
///     }
/// }
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Camera {
    /// Unique identifier for the camera
    ///
    /// This ID is used when making API calls to update or retrieve
    /// specific camera information.
    pub id: String,

    /// Human-readable name of the camera
    ///
    /// This is the name you see in the UniFi Protect interface,
    /// typically describing the camera's location or purpose.
    pub name: String,

    /// Current recording settings for the camera
    pub recording_settings: RecordingSettings,
}

/// Recording configuration for a camera.
///
/// Contains the settings that control when and how the camera records video.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingSettings {
    /// The current recording mode for the camera
    pub mode: RecordingMode,
}

/// Defines when a camera should record video.
///
/// This enum represents the three main recording modes available
/// in UniFi Protect cameras.
///
/// # Examples
///
/// ```rust
/// use unifi_protect_client::models::camera::RecordingMode;
///
/// let mode = RecordingMode::Always;
/// match mode {
///     RecordingMode::Always => println!("Camera records 24/7"),
///     RecordingMode::Schedule => println!("Camera follows a schedule"),
///     RecordingMode::Never => println!("Camera doesn't record"),
/// }
/// ```
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordingMode {
    /// Record continuously, 24/7
    ///
    /// The camera will always be recording video, regardless of
    /// motion detection or scheduling settings.
    Always,

    /// Record based on a configured schedule
    ///
    /// The camera will only record during the time periods
    /// defined in its recording schedule.
    Schedule,

    /// Never record video
    ///
    /// The camera will not record any video, though it may
    /// still detect motion and send notifications.
    Never,
}

/// Structure for updating camera settings.
///
/// This structure is used when making API calls to modify camera
/// configurations. All fields are optional, allowing for partial updates.
///
/// # Examples
///
/// ```rust
/// # use unifi_protect_client::models::camera::*;
/// #
/// // Enable continuous recording
/// let update = CameraUpdate {
///     recording_settings: Some(RecordingSettingsUpdate {
///         mode: Some(RecordingMode::Always),
///     }),
/// };
///
/// // Disable recording
/// let disable_update = CameraUpdate {
///     recording_settings: Some(RecordingSettingsUpdate {
///         mode: Some(RecordingMode::Never),
///     }),
/// };
///
/// // No changes (empty update)
/// let no_change = CameraUpdate {
///     recording_settings: None,
/// };
///
/// // Using builder pattern
/// let update = CameraUpdateBuilder::new()
///     .with_recording_settings(
///         RecordingSettingsUpdateBuilder::new()
///             .with_mode(RecordingMode::Always)
///             .build()
///     )
///     .build();
/// ```
#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct CameraUpdate {
    /// Optional recording settings to update
    ///
    /// If `None`, recording settings will not be modified.
    /// If `Some`, the contained settings will be applied to the camera.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(optional)]
    pub recording_settings: Option<RecordingSettingsUpdate>,
}

/// Update structure for camera recording settings.
///
/// Used within `CameraUpdate` to specify which recording settings
/// should be modified.
#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct RecordingSettingsUpdate {
    /// Optional new recording mode
    ///
    /// If `Some`, the camera's recording mode will be changed to this value.
    /// If `None`, the recording mode will remain unchanged.
    #[builder(optional)]
    pub mode: Option<RecordingMode>,
}
