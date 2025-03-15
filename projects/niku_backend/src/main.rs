//! Backend in charge of making discovery possible on NIKU.

use log::debug;
use rand::seq::IndexedRandom;
use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Json, Path, State},
    routing::{get, put},
    Router,
};
use log::error;
use niku_core::{set_logging, UploadTicket};
use thiserror::Error;

struct SharedData {
    tickets: HashMap<String, UploadTicket>,
}

#[derive(Error, Debug)]
enum RunError {
    #[error("Binding to the TCP listening port failed: {0}")]
    BingTcpListenerFailed(#[source] io::Error),
}

async fn run() -> Result<(), RunError> {
    set_logging();

    let state = Arc::new(Mutex::new(SharedData {
        tickets: HashMap::new(),
    }));

    let app = Router::new()
        .route("/files", put(put_files))
        .route("/files/{id}", get(get_files_id))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000")
        .await
        .map_err(RunError::BingTcpListenerFailed)?;

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn get_random_word() -> String {
    let words = ["hi", "this", "is", "a", "test!"];

    words
        .choose(&mut rand::rng())
        .expect("The vector will never be empty")
        .to_string()
}

async fn put_files(
    State(state): State<Arc<Mutex<SharedData>>>,
    Json(upload_ticket): Json<UploadTicket>,
) -> String {
    let tickets = &mut state.lock().unwrap().tickets;

    let mut word = get_random_word();

    loop {
        if !tickets.contains_key(&word) {
            break;
        }

        word = get_random_word();
    }

    tickets.insert(word.clone(), upload_ticket);

    word
}

async fn get_files_id(
    State(state): State<Arc<Mutex<SharedData>>>,
    Path(id): Path<String>,
) -> Json<UploadTicket> {
    let tickets = &mut state.lock().unwrap().tickets;

    Json(tickets.get(&id).unwrap().clone())
}

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => (),
        Err(err) => error!("{err}"),
    }
}
