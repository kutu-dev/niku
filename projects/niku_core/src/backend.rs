// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0



//! Shared structs use to communicate with the backend.

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
/// Relevant metadata about the state of the uploaded object on the backend server.
pub struct RegisteredObjectData {
    /// The ID of the object.
    pub id: String,

    /// A private UUIDv4 that must be used on a [ObjectKeepAliveRequest] to avoid the backend server deleting the object entry.
    pub keep_alive_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Request that can be send to the backend server the avoid it deleting the object entry.
pub struct ObjectKeepAliveRequest {
    /// The private UUIDv4 that has been returned by [RegisteredObjectData] used to identify and authenticate the object refresh.
    pub keep_alive_key: String,
}

#[derive(Serialize)]
/// Data that is returned when the server has an error.
pub struct ErrorResponse {
    /// The associated code with the raised error.
    pub code: String,

    /// A helpful message about the error that has occurred.
    pub message: String,
}

impl ErrorResponse {
    /// Crates a new [ErrorResponse]
    pub fn new(code: String, message: String) -> ErrorResponse {
        ErrorResponse { code, message }
    }
}
