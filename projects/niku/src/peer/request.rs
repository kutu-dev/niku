// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use anyhow::Result;
use reqwest::{Method, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::{Peer, PeerError};
use crate::backend::ErrorResponse;

impl Peer {
    pub(super) async fn request<T: Serialize>(
        &self,
        method: Method,
        path: &str,
        json: Option<&T>,
        with_address: Option<String>,
    ) -> Result<Response, PeerError> {
        let address = if let Some(address) = with_address {
            address
        } else {
            crate::get_recommended_backend_address()
        };

        let request = self.client.request(method, format!("{address}/{path}"));

        let request = if let Some(json) = json {
            request.json(json)
        } else {
            request
        };

        request.send().await.map_err(PeerError::PublishObjectFailed)
    }

    pub(super) async fn request_expect_json<T, S>(
        &self,
        method: Method,
        path: &str,
        json: Option<&T>,
        with_address: Option<String>,
    ) -> Result<S, PeerError>
    where
        T: Serialize,
        S: DeserializeOwned,
    {
        let response = self
            .request(method, path, json, with_address)
            .await?
            .bytes()
            .await
            .map_err(PeerError::PublishObjectFailed)?;

        let expected_json = serde_json::from_slice(response.as_ref());

        if let Ok(expected_json) = expected_json {
            return Ok(expected_json);
        }

        let backend_error: ErrorResponse = serde_json::from_slice(response.as_ref())?;

        Err(PeerError::BackendError(backend_error))
    }
}
