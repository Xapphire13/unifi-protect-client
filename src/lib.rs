use std::cell::RefCell;

use reqwest::{Client, ClientBuilder, StatusCode, header::HeaderMap};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use thiserror::Error;

use crate::auth::AuthCredentials;

pub mod api;
mod auth;
pub mod models;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("{0}")]
    NetworkError(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("deserialization error: {0}")]
    DeserializationError(String),
    #[error("unknown error")]
    Unknown,
}

pub struct UnifiProtectClient {
    client: RefCell<Option<Client>>,
    host: String,
    credentials: AuthCredentials,
}

impl UnifiProtectClient {
    pub fn new(host: &str, username: &str, password: &str) -> UnifiProtectClient {
        UnifiProtectClient {
            client: RefCell::new(None),
            host: host.to_owned(),
            credentials: AuthCredentials {
                username: username.to_owned(),
                password: password.to_owned(),
            },
        }
    }

    async fn ensure_authenticated(&self) -> Result<(), RequestError> {
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
            .unwrap()
            .post(url)
            .json(&json!({
                "username": self.credentials.username,
                "password": self.credentials.password,
                "rememberMe": true,
                "token": ""
            }))
            .send()
            .await
            .map_err(|err| RequestError::NetworkError(err.to_string()))?;

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

    async fn make_get_request<T: DeserializeOwned>(&self, uri: &str) -> Result<T, RequestError> {
        self.ensure_authenticated().await?;

        let url = format!("{}/{uri}", self.host);
        let response = { self.client.borrow().as_ref().unwrap().get(&url).send() }
            .await
            .map_err(|err| RequestError::NetworkError(err.to_string()))?;

        if !response.status().is_success() {
            return match response.status() {
                StatusCode::UNAUTHORIZED => Err(RequestError::Unauthorized),
                _ => Err(RequestError::Unknown),
            };
        }

        let result: T = response
            .json()
            .await
            .map_err(|err| RequestError::DeserializationError(err.to_string()))?;

        Ok(result)
    }

    async fn make_patch_request<T: Serialize>(
        &self,
        uri: &str,
        body: T,
    ) -> Result<(), RequestError> {
        self.ensure_authenticated().await?;

        let url = format!("{}/{uri}", self.host);
        let response = {
            self.client
                .borrow()
                .as_ref()
                .unwrap()
                .patch(&url)
                .json(&body)
                .send()
        }
        .await
        .map_err(|err| RequestError::NetworkError(err.to_string()))?;

        if !response.status().is_success() {
            return match response.status() {
                StatusCode::UNAUTHORIZED => Err(RequestError::Unauthorized),
                _ => Err(RequestError::Unknown),
            };
        }

        Ok(())
    }
}
