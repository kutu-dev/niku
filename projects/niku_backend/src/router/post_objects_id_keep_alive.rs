// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use axum::extract::{Json, Path, State};
use niku::backend::{ErrorResponse, ObjectKeepAliveRequest};
use niku::object::ObjectEntry;
use tokio::sync::Mutex;

use crate::errors::ServerError;
use crate::router::create_object_delete_task;
use crate::{KeepAliveEntry, SharedData};

#[utoipa::path(
    post,
    path = "/objects/{id}/keep-alive",
    params(("id" = String, Path, description = "The ID of the object.")),
    request_body = ObjectKeepAliveRequest,
    responses((status = OK, body = ObjectEntry), (status = NOT_FOUND, body = ErrorResponse))
)]
/// Request the server to keep the object alive.
pub(super) async fn post_objects_id_keep_alive(
    State(locked_state): State<Arc<Mutex<SharedData>>>,
    Path(id): Path<String>,
    Json(keep_alive_request): Json<ObjectKeepAliveRequest>,
) -> Result<(), ServerError> {
    let mut state = locked_state.lock().await;

    let keep_alive_entry = state
        .keep_alive_entries
        .get(&keep_alive_request.keep_alive_key)
        .ok_or(ServerError::UnknownKeepAliveKey)?;

    keep_alive_entry.delete_task.abort();
    let object_id = keep_alive_entry.object_id.clone();

    // Drop the reference
    let _ = keep_alive_entry;

    let delete_task = create_object_delete_task(
        locked_state.clone(),
        &id,
        &keep_alive_request.keep_alive_key,
    );

    state.keep_alive_entries.insert(
        keep_alive_request.keep_alive_key,
        KeepAliveEntry {
            object_id,
            delete_task,
        },
    );

    Ok(())
}
