// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use niku::backend::ErrorResponse;

pub(crate) enum ServerError {
    UnknownObject,
    UnknownKeepAliveKey,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        // TODO: Change this to use a map and avoid all that from()s
        let (status, code, message) = match self {
            ServerError::UnknownObject => (
                StatusCode::NOT_FOUND,
                String::from("0001@NKBE"),
                String::from("The requested object is not available"),
            ),

            ServerError::UnknownKeepAliveKey => (
                StatusCode::NOT_FOUND,
                String::from("0002@NKBE"),
                String::from("The given keep alive key doesn't match for any registered object"),
            ),
        };

        (status, Json(ErrorResponse::new(code, message))).into_response()
    }
}
