use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Camera {
    pub id: String,
    pub name: String,
    pub recording_settings: RecordingSettings,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingSettings {
    pub mode: RecordingMode,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordingMode {
    Always,
    Schedule,
    Never,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recording_settings: Option<RecordingSettingsUpdate>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingSettingsUpdate {
    pub mode: Option<RecordingMode>,
}
