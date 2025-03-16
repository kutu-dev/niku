//! Backend in charge of making discovery possible on NIKU.

mod router;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::{env, io};

use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use const_format::formatcp;
use niku_core::ObjectEntry;
use serde::Serialize;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info};

const ENV_VARS_PREFIX: &str = "NIKU_BACKEND_";

const OBJECT_ID_PREFIX_ENV_VAR_NAME: &str = formatcp!("{ENV_VARS_PREFIX}OBJECT_ID_PREFIX");
const DEFAULT_OBJECT_ID_PREFIX: &str = "test";

const SERVE_ADDRESS: &str = "0.0.0.0:4000";

#[cfg(debug_assertions)]
const OBJECT_LIFETIME_SECONDS: u64 = 5;

#[cfg(not(debug_assertions))]
const OBJECT_LIFETIME_SECONDS: u64 = 5 * 60;

const NOUNS_JSON: &str = include_str!("nouns.json");
const ADJECTIVES_JSON: &str = include_str!("adjectives.json");
const VERBS_JSON: &str = include_str!("verbs.json");

static NOUNS: LazyLock<Vec<String>> = LazyLock::new(|| {
    #[allow(clippy::expect_used)]
    serde_json::from_str(NOUNS_JSON).expect("The nouns.json file is invalid!")
});

static ADJECTIVES: LazyLock<Vec<String>> = LazyLock::new(|| {
    #[allow(clippy::expect_used)]
    serde_json::from_str(ADJECTIVES_JSON).expect("The adjectives.json file is invalid!")
});

static VERBS: LazyLock<Vec<String>> = LazyLock::new(|| {
    #[allow(clippy::expect_used)]
    serde_json::from_str(VERBS_JSON).expect("The verbs.json file is invalid!")
});

pub(crate) enum ServerError {
    UnknownObject,
    UnknownKeepAliveKey,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        // How we want errors responses to be serialized
        #[derive(Serialize)]
        struct ErrorResponse {
            code: String,
            message: String,
        }

        let (status, code, message) = match self {
            ServerError::UnknownObject => (
                StatusCode::NOT_FOUND,
                String::from("NKBE:1"),
                String::from("The requested object is not available"),
            ),

            ServerError::UnknownKeepAliveKey => (
                StatusCode::NOT_FOUND,
                String::from("NKBE:2"),
                String::from("The given keep alive key doesn't match for any registered object"),
            ),
        };

        (status, Json(ErrorResponse { code, message })).into_response()
    }
}

#[derive(Debug)]
struct KeepAliveEntry {
    ticket_id: String,
    delete_task: JoinHandle<()>,
}

struct SharedData {
    objects: HashMap<String, ObjectEntry>,
    keep_alive_entries: HashMap<String, KeepAliveEntry>,
    object_id_prefix: String,
}

impl SharedData {
    fn new(object_id_prefix: String) -> SharedData {
        SharedData {
            objects: HashMap::new(),
            keep_alive_entries: HashMap::new(),
            object_id_prefix,
        }
    }
}

#[derive(Error, Debug)]
/// Errors that may happen when running the server.
pub enum RunError {
    /// Binding to the TCP listening port failed
    #[error("Binding to the TCP listening port failed: {0}")]
    BingTcpListenerFailed(#[source] io::Error),

    /// Unable to start serving the server with Axum
    #[error("Unable to start serving the server with Axum: {0}")]
    ServeFailed(#[source] io::Error),
}

/// Start running the server
pub async fn run() -> Result<(), RunError> {
    let object_id_prefix =
        env::var(OBJECT_ID_PREFIX_ENV_VAR_NAME).unwrap_or(String::from(DEFAULT_OBJECT_ID_PREFIX));

    info!("Starting NIKU backend server...");
    info!("Object lifetime: {OBJECT_LIFETIME_SECONDS}s");
    info!("Object ID prefix: {object_id_prefix}");
    info!("Serving at http://{SERVE_ADDRESS}/");

    let shared_state = Arc::new(Mutex::new(SharedData::new(object_id_prefix)));
    let app = router::create_router(shared_state);

    let listener = tokio::net::TcpListener::bind(SERVE_ADDRESS)
        .await
        .map_err(RunError::BingTcpListenerFailed)?;

    axum::serve(listener, app)
        .await
        .map_err(RunError::ServeFailed)?;

    Ok(())
}
