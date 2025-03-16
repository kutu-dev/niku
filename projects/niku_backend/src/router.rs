use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Json, MatchedPath, Path, Request, State};
use axum::routing::{get, post, put};
use axum::Router;
use niku_core::{ObjectEntry, ObjectKeepAliveRequest, RegisteredObjectData};
use rand::seq::IndexedRandom;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time;
use tower_http::trace::TraceLayer;
use tracing::{info, trace};
use uuid::Uuid;

use crate::{KeepAliveEntry, ServerError, SharedData, ADJECTIVES, NOUNS, VERBS};

pub(crate) fn create_router(state: Arc<Mutex<SharedData>>) -> Router {
    Router::new()
        .route("/objects", put(put_objects))
        .route("/objects/{id}", get(get_objects_id))
        .route("/objects/{id}/keep-alive", post(post_objects_id_keep_alive))
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
}

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

/// Get a random value from a `&[&str]`
///
/// # Safety
/// The given slice must not be empty.
unsafe fn get_random_value_from_string_vec(values: &[String]) -> String {
    #[allow(clippy::expect_used)]
    values
        .choose(&mut rand::rng())
        .expect("The vector should never be empty")
        .to_string()
}

fn get_random_word(prefix: &str) -> String {
    unsafe {
        let adjective = get_random_value_from_string_vec(&ADJECTIVES);
        let noun = get_random_value_from_string_vec(&NOUNS);
        let verb = get_random_value_from_string_vec(&VERBS);

        format!("{prefix} {adjective} {noun} {verb}")
    }
}

async fn put_objects(
    State(locked_state): State<Arc<Mutex<SharedData>>>,
    Json(upload_ticket): Json<ObjectEntry>,
) -> Json<RegisteredObjectData> {
    let state = &mut locked_state.lock().await;

    // Iterate over until a unique ID is found, given the number of combinations
    // this should not happen more than one or two times at most
    let id = loop {
        let new_id = get_random_word(&state.object_id_prefix);

        if !state.objects.contains_key(&new_id) {
            break new_id;
        }
    };

    let keep_alive_key = Uuid::new_v4().to_string();

    state.objects.insert(id.clone(), upload_ticket);

    state.keep_alive_entries.insert(
        keep_alive_key.clone(),
        KeepAliveEntry {
            ticket_id: id.clone(),
            delete_task: create_object_delete_task(locked_state.clone(), &id, &keep_alive_key),
        },
    );

    if cfg!(debug_assertions) {
        info!(%id, %keep_alive_key, "Created new object");
    }

    Json(RegisteredObjectData { id, keep_alive_key })
}

async fn get_objects_id(
    State(state): State<Arc<Mutex<SharedData>>>,
    Path(id): Path<String>,
) -> Result<Json<ObjectEntry>, ServerError> {
    let objects = &mut state.lock().await.objects;
    let entry = objects.get(&id).ok_or(ServerError::UnknownObject)?.clone();

    if cfg!(debug_assertions) {
        info!(?entry, "Requested object entry");
    }

    Ok(Json(entry))
}

async fn post_objects_id_keep_alive(
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
    let ticket_id = keep_alive_entry.ticket_id.clone();

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
            ticket_id,
            delete_task,
        },
    );

    Ok(())
}
