use std::cell::RefCell;

use reqwest::{Client, StatusCode};
use secrecy::SecretString;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

use crate::auth::AuthCredentials;

pub mod api;
mod auth;
pub mod models;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("unauthorized")]
    Unauthorized,
    #[error("Deserialization error: {0}")]
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
                username: SecretString::from(username),
                password: SecretString::from(password),
            },
        }
    }

    async fn make_get_request<T: DeserializeOwned>(&self, uri: &str) -> Result<T, RequestError> {
        self.ensure_authenticated().await?;

        let url = format!("{}/{uri}", self.host);
        let response = { self.client.borrow().as_ref().unwrap().get(&url).send() }
            .await
            .map_err(RequestError::NetworkError)?;

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
        .map_err(RequestError::NetworkError)?;

        match response.status() {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(RequestError::Unauthorized),
            _ if !response.status().is_success() => Err(RequestError::Unknown),
            _ => Ok(()),
        }
    }
}
