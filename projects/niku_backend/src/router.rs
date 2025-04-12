// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

mod get_objects_id;
mod post_objects_id_keep_alive;
mod put_objects;

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{MatchedPath, Request};
use axum::Router;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time;
use tower_http::trace::TraceLayer;
use tracing::{info, trace};
use utoipa::openapi::{Info, License};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

use crate::router::get_objects_id::*;
use crate::router::post_objects_id_keep_alive::*;
use crate::router::put_objects::*;
use crate::SharedData;

pub(crate) fn create_router(state: Arc<Mutex<SharedData>>) -> Router {
    let (router, mut spec) = OpenApiRouter::new()
        .routes(routes!(
            put_objects,
            get_objects_id,
            post_objects_id_keep_alive
        ))
        .with_state(state)
        .layer(TraceLayer::new_for_http().make_span_with(|req: &Request| {
            let method = req.method();
            let uri = req.uri();

            let matched_path = req
                .extensions()
                .get::<MatchedPath>()
                .map(|matched_path| matched_path.as_str());

            tracing::debug_span!("request", %method, %uri, matched_path)
        }))
        .split_for_parts();

    if !cfg!(debug_assertions) {
        return router;
    }

    info!("Debug mode enabled, spinning up OpenAPI documentation at '/swagger'");

    spec.info = Info::builder()
        .title("NIKU Backend API")
        .description(Some(
            "The server used to interchange between peers their IDs given a human friendly name.",
        ))
        .version("1.0.0")
        .license(Some(
            License::builder()
                .name("Mozilla Public License 2.0")
                .identifier(Some("MPL-2.0"))
                .build(),
        ))
        .build();

    router.merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", spec.clone()))
}

/// Creates the background task responsible of deleting the object at the end of its lifetime.
fn create_object_delete_task(
    locked_state: Arc<Mutex<SharedData>>,
    id: &str,
    keep_alive_key: &str,
) -> JoinHandle<()> {
    // Only on debug mode for privacy reasons
    if cfg!(debug_assertions) {
        trace!(%id, %keep_alive_key, "Creating an object scheduled delete task");
    }

    let locked_state = locked_state.clone();
    let object_id = String::from(id);
    let object_keep_alive_key = String::from(keep_alive_key);

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));
        // The first tick is immediate, skip it
        interval.tick().await;
        interval.tick().await;

        let mut state = locked_state.lock().await;

        // Only on debug mode for privacy reasons
        if cfg!(debug_assertions) {
            info!(
                keep_alive_key = object_keep_alive_key,
                "Object '{object_id}' timed out! Deleting it..."
            );
        }

        state.objects.remove(&object_id);
        state.keep_alive_entries.remove(&object_keep_alive_key);
    })
}
