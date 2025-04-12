// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use axum::extract::{Json, State};
use niku::backend::RegisteredObjectData;
use niku::object::ObjectEntry;
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;

use crate::extensions::StringSliceExt;
use crate::router::create_object_delete_task;
use crate::{KeepAliveEntry, SharedData, ADJECTIVES, NOUNS, VERBS};

#[utoipa::path(put, path = "/objects", request_body = ObjectEntry, responses((status = OK, body = RegisteredObjectData)))]
/// Send a new object to be registered.
///
/// Registerer an object, returns the data needed to retrieve it from an external peer
/// and the key that must be send to avoid the server to remove it.
pub(super) async fn put_objects(
    State(locked_state): State<Arc<Mutex<SharedData>>>,
    Json(upload_ticket): Json<ObjectEntry>,
) -> Json<RegisteredObjectData> {
    let state = &mut locked_state.lock().await;

    // Iterate over until a unique ID is found, given the number of combinations
    // this should not happen more than one or two times at most
    let id = loop {
        let new_id = unsafe {
            let adjective = ADJECTIVES.get_random();
            let noun = NOUNS.get_random();
            let verb = VERBS.get_random();

            format!("{}-{adjective}-{noun}-{verb}", &state.object_id_prefix)
        };

        if !state.objects.contains_key(&new_id) {
            break new_id;
        }
    };

    let keep_alive_key = Uuid::new_v4().to_string();

    state.objects.insert(id.clone(), upload_ticket);

    state.keep_alive_entries.insert(
        keep_alive_key.clone(),
        KeepAliveEntry {
            object_id: id.clone(),
            delete_task: create_object_delete_task(locked_state.clone(), &id, &keep_alive_key),
        },
    );

    if cfg!(debug_assertions) {
        info!(%id, %keep_alive_key, "Created new object");
    }

    Json(RegisteredObjectData { id, keep_alive_key })
}
