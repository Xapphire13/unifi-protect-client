//! Authentication handling for the UniFi Protect API.
//!
//! This module contains the authentication logic and credential management
//! for connecting to UniFi Protect controllers.

use reqwest::{Client, ClientBuilder, StatusCode, header::HeaderMap};
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;

use crate::{RequestError, UnifiProtectClient};

/// Secure storage for authentication credentials.
///
/// This structure holds the username and password required for
/// UniFi Protect authentication. Credentials are stored using
/// the `secrecy` crate to prevent accidental exposure in logs.
pub struct AuthCredentials {
    /// Username for authentication (securely stored)
    pub username: SecretString,

    /// Password for authentication (securely stored)
    pub password: SecretString,
}

impl UnifiProtectClient {
    /// Ensures the client is authenticated before making API requests.
    ///
    /// This method checks if an authenticated HTTP client exists, and if not,
    /// performs the authentication process to obtain the necessary tokens
    /// and cookies for subsequent API calls.
    ///
    /// # Internal Use
    ///
    /// This method is for internal use by the client and is called automatically
    /// before each API request.
    pub(crate) async fn ensure_authenticated(&self) -> Result<(), RequestError> {
        // If we have a client, we're already authenticated
        if self.client.lock().unwrap().is_some() {
            return Ok(());
        }

        self.authenticate().await
    }

    /// Aquires authentication headers and initializes an authenticated HTTP client
    ///
    /// # Internal Use
    pub(crate) async fn authenticate(&self) -> Result<(), RequestError> {
        let headers = self.acquire_auth_headers().await?;

        *self.client.lock().unwrap() = Some(
            ClientBuilder::new()
                .danger_accept_invalid_certs(true)
                .default_headers(headers)
                .build()
                .map_err(|_| RequestError::Unknown)?,
        );

        Ok(())
    }

    /// Performs login and acquires authentication headers.
    ///
    /// This method handles the UniFi Protect login process, which involves
    /// sending credentials to the login endpoint and extracting the required
    /// CSRF token and session cookies from the response.
    ///
    /// # Authentication Flow
    ///
    /// The UniFi Protect API uses a combination of:
    /// - CSRF tokens for request validation
    /// - Session cookies for maintaining authenticated state
    ///
    /// Both are required for successful API requests after login.
    ///
    /// # Returns
    ///
    /// Returns a `HeaderMap` containing the authentication headers needed
    /// for subsequent API requests.
    ///
    /// # Errors
    ///
    /// Returns `RequestError::Unauthorized` if login fails due to invalid
    /// credentials, or other `RequestError` variants for network or parsing issues.
    async fn acquire_auth_headers(&self) -> Result<HeaderMap, RequestError> {
        let url = format!("{}/api/auth/login", self.host);
        let response = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(RequestError::NetworkError)?
            .post(url)
            .json(&json!({
                "username": self.credentials.username.expose_secret(),
                "password": self.credentials.password.expose_secret(),
            }))
            .send()
            .await
            .map_err(RequestError::NetworkError)?;

        if !response.status().is_success() {
            return match response.status() {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(RequestError::Unauthorized),
                _ => Err(RequestError::Unknown),
            };
        }

        let response_headers = response.headers();
        let mut headers = HeaderMap::new();

        // Extract CSRF token
        headers.insert(
            "X-CSRF-Token",
            response_headers
                .get("X-CSRF-Token")
                .ok_or(RequestError::Unknown)?
                .clone(),
        );

        // Extract session cookie
        headers.insert(
            "Cookie",
            response_headers
                .get("Set-Cookie")
                .ok_or(RequestError::Unknown)?
                .clone(),
        );

        Ok(headers)
    }
}
