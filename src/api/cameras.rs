use crate::{
    RequestError, UnifiProtectClient,
    models::camera::{Camera, CameraUpdate},
};

impl UnifiProtectClient {
    pub async fn list_cameras(&self) -> Result<Vec<Camera>, RequestError> {
        let cameras = self.make_get_request("proxy/protect/api/cameras").await?;

        Ok(cameras)
    }

    pub async fn update_camera(
        &self,
        camera_id: &str,
        updates: CameraUpdate,
    ) -> Result<(), RequestError> {
        let uri = format!("proxy/protect/api/cameras/{camera_id}");
        self.make_patch_request(&uri, updates).await
    }
}
