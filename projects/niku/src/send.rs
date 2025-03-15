use core::error;
use std::io;
use std::path::PathBuf;

use iroh::protocol::Router;
use iroh_blobs::rpc::client::blobs::{MemClient, WrapOption};
use iroh_blobs::util::SetTagOption;
use log::{debug, info};
use niku_core::UploadTicket;
use reqwest::Client;
use thiserror::Error;

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

    let ticket = UploadTicket {
        node_addr: router.endpoint().node_addr().await?,
        file_hash: blob.hash,
    };

    debug!("Uploading ticket: {ticket:?}");

    let id = client
        .put("http://localhost:4000/files")
        .json(&ticket)
        .send()
        .await
        .map_err(SendError::BackendRequestFailed)?
        .text()
        .await
        .map_err(SendError::BackendRequestFailed)?;

    info!("Your file ID is: {id}");

    tokio::signal::ctrl_c()
        .await
        .map_err(SendError::WaitOnCtrlCFailed)?;

    Ok(())
}
