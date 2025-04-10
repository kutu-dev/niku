// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0



//! Backend in charge of making discovery possible on NIKU.

mod errors;
mod router;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::{env, io};

use const_format::formatcp;
use log::warn;
use niku_core::object::ObjectEntry;
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

macro_rules! parse_word_list_json {
    ($name:ident, $path:literal) => {
        /// Static list of words.
        pub static $name: LazyLock<Vec<String>> = LazyLock::new(|| {
            serde_json::from_str(include_str!($path)).expect("Parsing the list of words failed")
        });
    };
}

parse_word_list_json!(NOUNS, "data/nouns.json");
parse_word_list_json!(ADJECTIVES, "data/adjectives.json");
parse_word_list_json!(VERBS, "data/verbs.json");

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

    if cfg!(debug_assertions) {
        warn!("DEBUG MODE ENABLED! Private information may be exposed!")
    }

    info!("Object lifetime: {OBJECT_LIFETIME_SECONDS}s");
    info!("Object ID prefix: {object_id_prefix}");
    info!("Serving at http://{SERVE_ADDRESS}/");

    let state = Arc::new(Mutex::new(SharedData::new(object_id_prefix)));
    let router = router::create_router(state);

    let listener = tokio::net::TcpListener::bind(SERVE_ADDRESS)
        .await
        .map_err(RunError::BingTcpListenerFailed)?;

    axum::serve(listener, router)
        .await
        .map_err(RunError::ServeFailed)?;

    Ok(())
}
