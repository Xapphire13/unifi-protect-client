use reqwest::{Client, ClientBuilder, header::HeaderMap};
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;

use crate::{RequestError, UnifiProtectClient};

pub struct AuthCredentials {
    pub username: SecretString,
    pub password: SecretString,
}

impl UnifiProtectClient {
    pub(crate) async fn ensure_authenticated(&self) -> Result<(), RequestError> {
        // If we have a client, we're already authenticated
        if self.client.borrow().is_some() {
            return Ok(());
        }

        // let csrf_token = self.aquire_csrf_token().await?;
        let headers = self.acquire_auth_headers().await?;

        self.client.replace(Some(
            ClientBuilder::new()
                .danger_accept_invalid_certs(true)
                .default_headers(headers)
                .build()
                .map_err(|_| RequestError::Unknown)?,
        ));

        Ok(())
    }

    /// Performs a login request and returns the headers required for
    /// authorized requests
    ///
    /// The protect API uses a combination of both a CSRF token and cookies for
    /// authorization. .
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
            return Err(RequestError::Unknown);
        }

        let response_headers = response.headers();
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-CSRF-Token",
            response_headers
                .get("X-CSRF-Token")
                .ok_or(RequestError::Unknown)?
                .clone(),
        );
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
