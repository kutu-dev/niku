use core::error;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use iroh::protocol::Router;
use iroh_blobs::rpc::client::blobs::{MemClient, WrapOption};
use iroh_blobs::util::SetTagOption;
use log::debug;
use niku_core::{ObjectEntry, ObjectKeepAliveRequest, RegisteredObjectData};
use reqwest::Client;
use thiserror::Error;
use tokio::time;

use crate::BASE_BACKEND_URL;

#[derive(Error, Debug)]
pub enum SendError {
    #[error("Canonicalize the file path failed: {0}")]
    CanonicalizePathFailed(#[source] io::Error),

    #[error("An Iroh error has occurred: {0}")]
    IrohError(#[from] anyhow::Error),

    #[error("Unable to send the file ticket to the server: {0}")]
    BackendRequestFailed(#[source] reqwest::Error),

    #[error("The given file is not a plain file or a folder")]
    InvalidFileKind,

    #[error("The given file or folder doesn't have a Unicode filename")]
    NotUnicodeFilename,
}

/// Creates a new object entry for a file.
///
/// # Safety
/// Doesn't check if the given path is for a file.
async unsafe fn create_file_object_entry(
    path: &PathBuf,
    blobs_client: &MemClient,
    router: &Router,
) -> Result<ObjectEntry, SendError> {
    let blob = blobs_client
        .add_from_path(path.clone(), true, SetTagOption::Auto, WrapOption::NoWrap)
        .await?
        .finish()
        .await?;

    let file_name = path
        .file_name()
        .expect("The path is always for a real file")
        .to_str()
        .ok_or(SendError::NotUnicodeFilename)?
        .to_string();

    println!("📤 Sending file '{file_name}'");

    Ok(ObjectEntry {
        node_address: router.endpoint().node_addr().await?,
        file_hash: blob.hash,
        kind: niku_core::ObjectKind::File { name: file_name },
    })
}

/*
async fn create_directory_object_entry(
    path: &PathBuf,
    blobs_client: &MemClient,
    router: &Router,
) -> Result<ObjectEntry, SendError> {


}
*/

pub(crate) async fn send(
    path: &str,
    client: Client,
    blobs_client: &MemClient,
    router: &Router,
) -> Result<(), SendError> {
    let object = if path == "-" {
        todo!("TEXT MODE");
    } else {
        let path = PathBuf::from(path)
            .canonicalize()
            .map_err(SendError::CanonicalizePathFailed)?;

        if path.is_file() {
            unsafe { create_file_object_entry(&path, blobs_client, router).await }
        } else {
            Err(SendError::InvalidFileKind)
        }
    }?;

    debug!("Uploading object: {object:?}");

    let registration_data = client
        .put(format!("{BASE_BACKEND_URL}/objects"))
        .json(&object)
        .send()
        .await
        .map_err(SendError::BackendRequestFailed)?
        .json::<RegisteredObjectData>()
        .await
        .map_err(SendError::BackendRequestFailed)?;

    println!("  Your ID is: '{}'", registration_data.id);
    println!();
    println!("📥 On the other device, please run:");
    println!("  niku receive '{}'", registration_data.id);
    println!();
    println!("🌐 Or use one of the official GUI apps:");
    println!("  https://niku.app/download");

    let mut interval = time::interval(Duration::from_secs(1));

    loop {
        debug!("Keeping alive the connection!");

        interval.tick().await;
        client
            .post(format!(
                "{BASE_BACKEND_URL}/objects/{}/keep-alive",
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
