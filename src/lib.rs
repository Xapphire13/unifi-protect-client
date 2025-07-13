//! # UniFi Protect Client
//!
//! A Rust client library for interacting with the UniFi Protect API.
//!
//! This crate provides a simple and type-safe way to interact with UniFi Protect
//! cameras and other devices through the REST API.
//!
//! ## Features
//!
//! - Secure credential handling using the `secrecy` crate
//! - Automatic authentication and session management
//! - Type-safe API responses with serde deserialization
//! - Support for camera management operations
//!
//! ## Quick Start
//!
//! ```rust
//! # use unifi_protect_client::UnifiProtectClient;
//! # use anyhow::Result;
//! #
//! # async fn example() -> Result<()> {
//! let client = UnifiProtectClient::new(
//!     "https://192.168.1.1",
//!     "username",
//!     "password"
//! );
//!
//! // List all cameras
//! let cameras = client.list_cameras().await?;
//! println!("Found {} cameras", cameras.len());
//! #
//! # Ok(())
//! # }
//! ```

use std::sync::{Arc, Mutex};

use reqwest::{Client, StatusCode};
use secrecy::SecretString;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

use crate::auth::AuthCredentials;

pub mod api;
mod auth;
pub mod models;

/// Errors that can occur when making requests to the UniFi Protect API.
///
/// This enum covers the various error conditions that may arise during
/// API interactions, including network failures, authentication issues,
/// and data parsing problems.
#[derive(Error, Debug)]
pub enum RequestError {
    /// Network-related errors (connection failures, timeouts, etc.)
    ///
    /// This error wraps underlying `reqwest::Error` types and indicates
    /// issues with the HTTP transport layer.
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// Authentication or authorization failures
    ///
    /// Returned when the API responds with HTTP 401 (Unauthorized) or
    /// HTTP 403 (Forbidden) status codes. This typically indicates
    /// invalid credentials or expired sessions.
    #[error("Unauthorized access - check your credentials")]
    Unauthorized,

    /// JSON deserialization errors
    ///
    /// Occurs when the API response cannot be parsed into the expected
    /// data structure. This might happen due to API changes or unexpected
    /// response formats.
    #[error("Failed to parse API response: {0}")]
    DeserializationError(String),

    /// Generic error for unhandled cases
    ///
    /// Used for HTTP errors that don't fall into other categories
    /// or unexpected error conditions.
    #[error("An unknown error occurred")]
    Unknown,
}

/// Client for interacting with the UniFi Protect API.
///
/// This is the main entry point for all UniFi Protect operations. The client
/// handles authentication, session management, and provides methods for
/// interacting with various UniFi Protect resources.
///
/// ## Example
///
/// ```rust
/// # use unifi_protect_client::UnifiProtectClient;
/// #
/// let client = UnifiProtectClient::new(
///     "https://192.168.1.1",  // Your UniFi Protect controller URL
///     "admin",                // Username
///     "password"              // Password
/// );
/// ```
pub struct UnifiProtectClient {
    client: Arc<Mutex<Option<Client>>>,
    host: String,
    credentials: AuthCredentials,
}

impl UnifiProtectClient {
    /// Creates a new UniFi Protect client.
    ///
    /// This constructor initializes a new client with the provided connection
    /// details and credentials. Authentication is performed lazily on the first
    /// API request.
    ///
    /// # Arguments
    ///
    /// * `host` - The base URL of your UniFi Protect controller (e.g., `https://192.168.1.1`)
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    ///
    /// # Security
    ///
    /// Credentials are stored securely using the `secrecy` crate and are only
    /// exposed during authentication requests.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use unifi_protect_client::UnifiProtectClient;
    /// #
    /// let client = UnifiProtectClient::new(
    ///     "https://192.168.1.1",
    ///     "admin",
    ///     "your-secure-password"
    /// );
    /// ```
    #[must_use]
    pub fn new(host: &str, username: &str, password: &str) -> UnifiProtectClient {
        UnifiProtectClient {
            client: Arc::new(Mutex::new(None)),
            host: host.to_owned(),
            credentials: AuthCredentials {
                username: SecretString::from(username),
                password: SecretString::from(password),
            },
        }
    }

    /// Makes a GET request to the specified API endpoint.
    ///
    /// This method automatically handles authentication and will attempt to
    /// re-authenticate once if the request fails with an authentication error.
    ///
    /// # Arguments
    ///
    /// * `path` - The API endpoint path (relative to the host)
    ///
    /// # Returns
    ///
    /// Returns the deserialized response on success, or a `RequestError` on failure.
    ///
    /// # Errors
    ///
    /// * `RequestError::Unauthorized` - Authentication failed, even after retry
    /// * `RequestError::NetworkError` - Network-related failures
    /// * `RequestError::DeserializationError` - Failed to parse response
    /// * `RequestError::Unknown` - Other HTTP errors
    async fn make_get_request<T: DeserializeOwned>(&self, path: &str) -> Result<T, RequestError> {
        self.ensure_authenticated().await?;

        let url = format!("{}/{path}", self.host);
        let mut retries_remaining = 1u8;

        while retries_remaining > 0 {
            let response = {
                self.client
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .get(&url)
                    .send()
            }
            .await
            .map_err(RequestError::NetworkError)?;

            if !response.status().is_success() {
                match response.status() {
                    StatusCode::UNAUTHORIZED => {
                        // Re-authenticate and try again if we haven't already retried
                        if retries_remaining > 0 {
                            retries_remaining -= 1;
                            self.authenticate().await?;
                            continue;
                        }

                        return Err(RequestError::Unauthorized);
                    }
                    _ => return Err(RequestError::Unknown),
                };
            }

            let result: T = response
                .json()
                .await
                .map_err(|err| RequestError::DeserializationError(err.to_string()))?;

            return Ok(result);
        }

        Err(RequestError::Unknown)
    }

    /// Makes a PATCH request to the specified API endpoint.
    ///
    /// This method automatically handles authentication and will attempt to
    /// re-authenticate once if the request fails with an authentication error.
    ///
    /// # Arguments
    ///
    /// * `path` - The API endpoint path (relative to the host)
    /// * `body` - The request body to serialize as JSON
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `RequestError` on failure.
    ///
    /// # Errors
    ///
    /// * `RequestError::Unauthorized` - Authentication failed, even after retry
    /// * `RequestError::NetworkError` - Network-related failures
    /// * `RequestError::Unknown` - Other HTTP errors
    async fn make_patch_request<T: Serialize>(
        &self,
        path: &str,
        body: T,
    ) -> Result<(), RequestError> {
        self.ensure_authenticated().await?;

        let url = format!("{}/{path}", self.host);
        let mut retries_remaining = 1u8;

        while retries_remaining > 0 {
            let response = {
                self.client
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .patch(&url)
                    .json(&body)
                    .send()
            }
            .await
            .map_err(RequestError::NetworkError)?;

            if !response.status().is_success() {
                match response.status() {
                    StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                        // Re-authenticate and try again if we haven't already retried
                        if retries_remaining > 0 {
                            retries_remaining -= 1;
                            self.authenticate().await?;
                            continue;
                        }

                        return Err(RequestError::Unauthorized);
                    }
                    _ => return Err(RequestError::Unknown),
                };
            }

            return Ok(());
        }

        Err(RequestError::Unknown)
    }
}
