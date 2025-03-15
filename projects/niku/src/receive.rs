use iroh_blobs::store::{ExportFormat, ExportMode};
use reqwest::Client;

use core::error;
use std::env::home_dir;
use std::io;
use std::path::PathBuf;

use iroh::protocol::Router;
use iroh_blobs::rpc::client::blobs::{MemClient, WrapOption};
use iroh_blobs::util::SetTagOption;
use log::{debug, info};
use niku_core::UploadTicket;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReceiveError {
    #[error("Unable to retrieve the file ticket to the server: {0}")]
    BackendRequestFailed(#[source] reqwest::Error),

    #[error("An unknown has occurred: {0}")]
    Unknown(#[from] anyhow::Error),
}

pub(crate) async fn receive(
    id: &str,
    client: Client,
    blobs_client: &MemClient,
) -> Result<(), ReceiveError> {
    let ticket = client
        .get(format!("http://localhost:4000/files/{id}"))
        .send()
        .await
        .map_err(ReceiveError::BackendRequestFailed)?
        .json::<UploadTicket>()
        .await
        .map_err(ReceiveError::BackendRequestFailed)?;

    println!("{ticket:?}");

    blobs_client
        .download(ticket.file_hash, ticket.node_addr.clone())
        .await?
        .finish()
        .await?;

    let mut path = home_dir().unwrap();
    path.push("TEST.txt");

    blobs_client
        .export(ticket.file_hash, path, ExportFormat::Blob, ExportMode::Copy)
        .await?
        .finish()
        .await?;

    Ok(())
}
