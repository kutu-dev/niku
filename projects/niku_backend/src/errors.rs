// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use niku_core::backend::ErrorResponse;

pub(crate) enum ServerError {
    UnknownObject,
    UnknownKeepAliveKey,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ServerError::UnknownObject => (
                StatusCode::NOT_FOUND,
                String::from("NKBE:0001"),
                String::from("The requested object is not available"),
            ),

            ServerError::UnknownKeepAliveKey => (
                StatusCode::NOT_FOUND,
                String::from("NKBE:0002"),
                String::from("The given keep alive key doesn't match for any registered object"),
            ),
        };

        (status, Json(ErrorResponse::new(code, message))).into_response()
    }
}
