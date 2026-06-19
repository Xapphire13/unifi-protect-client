# unifi-protect-client

[![Crates.io Version](https://img.shields.io/crates/v/unifi-protect-client)](https://crates.io/crates/unifi-protect-client)
[![docs.rs](https://img.shields.io/docsrs/unifi-protect-client)](https://docs.rs/unifi-protect-client)

A Rust client library for interacting with the UniFi Protect API.

## Features

- 🔄 Automatic authentication and session management
- 📝 Type-safe API responses with serde deserialization
- 📹 Camera management operations
- 🛡️ Comprehensive error handling

## Installation

```bash
cargo add unifi-protect-client
```

## Quick Start

```rust
use unifi_protect_client::{UnifiProtectClient, models::camera::*};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a client
    let client = UnifiProtectClient::new(
        "https://192.168.1.1",
        "username",
        "password"
    );

    // List all cameras
    let cameras = client.list_cameras().await?;
    println!("Found {} cameras", cameras.len());

    for camera in &cameras {
        println!("Camera: {} (ID: {})", camera.name, camera.id);
        println!("Recording mode: {:?}", camera.recording_settings.mode);
    }

    // Update a camera's recording mode
    if let Some(camera) = cameras.first() {
        let update = CameraUpdate {
            recording_settings: Some(RecordingSettingsUpdate {
                mode: Some(RecordingMode::Always),
            }),
        };

        client.update_camera(&camera.id, update).await?;
        println!("Updated camera {} to always record", camera.name);
    }

    Ok(())
}
```

## API Documentation

See <https://docs.rs/unifi-protect-client>

## Troubleshooting

### Common Issues

1. **Authentication Failures**
   ```
   Error: Unauthorized access - check your credentials
   ```
   - Verify username and password (See `Admins & Users` in your UniFi console)
   - Ensure the user has sufficient permissions (Must have `Protect` permissions in UniFi console)

2. **Network Connectivity**
   ```
   Error: Network error: connection refused
   ```
   - Verify the controller URL is correct
   - Check network connectivity
   - Ensure the controller is running and accessible

3. **Invalid Response Format**
   ```
   Error: Failed to parse API response: missing field 'id'
   ```
   - This may indicate API version compatibility issues
   - Check if your controller firmware is up to date

## Other UniFi Clients

This crate is for interacting with the UniFi Protect API.

- If you are looking for a client for the UniFi Access API checkout: [unifi_access](https://github.com/Carter12s/unifi_access)
- If you are looking for a client for the UniFi Network API checkout: [unifi-rs](https://github.com/CallumTeesdale/unifi-rs)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Releasing

Publishing to [crates.io](https://crates.io/crates/unifi-protect-client) is automated via GitHub Actions and triggered by pushing a version tag (`v*`). To cut a release:

1. Bump `version` in `Cargo.toml` to the new version (`X.Y.Z`).
2. Commit and merge that change to `main`.
3. Tag the release commit and push the tag, using the same version as `Cargo.toml`:

   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

The workflow checks that the tag matches the version in `Cargo.toml` (tag `vX.Y.Z` ↔ `version = "X.Y.Z"`) and fails the publish if they differ.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Disclaimer

This is an unofficial client library and is not affiliated with or endorsed by Ubiquiti Inc. UniFi and Protect are trademarks of Ubiquiti Inc.
