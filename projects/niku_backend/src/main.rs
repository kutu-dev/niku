//! Backend in charge of making discovery possible on NIKU.

use log::{debug, trace};
use rand::seq::IndexedRandom;
use std::{collections::HashMap, io, sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle, time};
use uuid::Uuid;

use axum::{
    extract::{Json, Path, State},
    routing::{get, post, put},
    Router,
};
use log::error;
use niku_core::{set_logging, ObjectKeepAliveRequest, ObjectRegistrationData, ObjectTicket};
use thiserror::Error;

#[derive(Debug)]
struct KeepAliveEntry {
    ticket_id: String,
    delete_task: JoinHandle<()>,
}

struct SharedData {
    tickets: HashMap<String, ObjectTicket>,
    keep_alive_entries: HashMap<String, KeepAliveEntry>,
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
        keep_alive_entries: HashMap::new(),
    }));

    let app = Router::new()
        .route("/files", put(put_files))
        .route("/files/{id}", get(get_files_id))
        .route("/files/{id}", post(post_files_id))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000")
        .await
        .map_err(RunError::BingTcpListenerFailed)?;

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn create_delete_task(
    locked_state: Arc<Mutex<SharedData>>,
    id: &str,
    keep_alive_key: &str,
) -> JoinHandle<()> {
    let locked_state = locked_state.clone();
    let object_id = String::from(id);
    let object_keep_alive_key = String::from(keep_alive_key);

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));
        // The first tick is immediate, skip it
        interval.tick().await;
        interval.tick().await;

        let mut state = locked_state.lock().await;

        println!("Deleting object {object_id}");
        state.tickets.remove(&object_id);
        state.keep_alive_entries.remove(&object_keep_alive_key);
    })
}

fn get_random_word() -> String {
    let words = ["hi", "this", "is", "a", "test!"];

    words
        .choose(&mut rand::rng())
        .expect("The vector will never be empty")
        .to_string()
}

async fn put_files(
    State(locked_state): State<Arc<Mutex<SharedData>>>,
    Json(upload_ticket): Json<ObjectTicket>,
) -> Json<ObjectRegistrationData> {
    let state = &mut locked_state.lock().await;

    let mut id = get_random_word();

    loop {
        if !state.tickets.contains_key(&id) {
            break;
        }

        id = get_random_word();
    }

    let keep_alive_key = Uuid::new_v4().to_string();

    state.tickets.insert(id.clone(), upload_ticket);

    state.keep_alive_entries.insert(
        keep_alive_key.clone(),
        KeepAliveEntry {
            ticket_id: id.clone(),
            delete_task: create_delete_task(locked_state.clone(), &id, &keep_alive_key),
        },
    );

    Json(ObjectRegistrationData { id, keep_alive_key })
}

async fn get_files_id(
    State(state): State<Arc<Mutex<SharedData>>>,
    Path(id): Path<String>,
) -> Json<ObjectTicket> {
    let tickets = &mut state.lock().await.tickets;

    Json(tickets.get(&id).unwrap().clone())
}

async fn post_files_id(
    State(locked_state): State<Arc<Mutex<SharedData>>>,
    Path(id): Path<String>,
    Json(keep_alive_request): Json<ObjectKeepAliveRequest>,
) {
    let mut state = locked_state.lock().await;

    let mut keep_alive_entry = state
        .keep_alive_entries
        .get(&keep_alive_request.keep_alive_key)
        .unwrap();

    keep_alive_entry.delete_task.abort();
    let ticket_id = keep_alive_entry.ticket_id.clone();

    // Drop the reference
    let _ = keep_alive_entry;

    let delete_task = create_delete_task(
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
}

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => (),
        Err(err) => error!("{err}"),
    }
}
