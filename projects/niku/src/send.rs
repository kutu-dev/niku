use core::error;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use iroh::protocol::Router;
use iroh_blobs::rpc::client::blobs::{MemClient, WrapOption};
use iroh_blobs::util::SetTagOption;
use log::debug;
use niku_core::{ObjectKeepAliveRequest, ObjectRegistrationData, ObjectTicket};
use reqwest::Client;
use thiserror::Error;
use tokio::time;

#[derive(Error, Debug)]
pub enum SendError {
    #[error("Canonicalize the file path failed: {0}")]
    CanonicalizePathFailed(#[source] io::Error),

    #[error("Unable to wait for file downloading, waiting for Ctrl-C failed: {0}")]
    WaitOnCtrlCFailed(#[source] io::Error),

    #[error("An unknown has occurred: {0}")]
    Unknown(#[from] anyhow::Error),

    #[error("Unable to send the file ticket to the server: {0}")]
    BackendRequestFailed(#[source] reqwest::Error),
}

pub(crate) async fn send(
    path: &str,
    client: Client,
    blobs_client: &MemClient,
    router: &Router,
) -> Result<(), SendError> {
    let path = PathBuf::from(path)
        .canonicalize()
        .map_err(SendError::CanonicalizePathFailed)?;

    let blob = blobs_client
        .add_from_path(path, true, SetTagOption::Auto, WrapOption::NoWrap)
        .await?
        .finish()
        .await?;

    let ticket = ObjectTicket {
        node_addr: router.endpoint().node_addr().await?,
        file_hash: blob.hash,
    };

    debug!("Uploading ticket: {ticket:?}");

    let registration_data = client
        .put("http://localhost:4000/files")
        .json(&ticket)
        .send()
        .await
        .map_err(SendError::BackendRequestFailed)?
        .json::<ObjectRegistrationData>()
        .await
        .map_err(SendError::BackendRequestFailed)?;

    println!(
        "Your file ID is: {} ({})",
        registration_data.id, registration_data.keep_alive_key
    );

    let mut interval = time::interval(Duration::from_secs(1));

    loop {
        debug!("Keeping alive the connection!");

        interval.tick().await;
        client
            .post(format!(
                "http://localhost:4000/files/{}",
                registration_data.id
            ))
            .json(&ObjectKeepAliveRequest {
                keep_alive_key: registration_data.keep_alive_key.clone(),
            })
            .send()
            .await
            .map_err(SendError::BackendRequestFailed)?;
    }
}
