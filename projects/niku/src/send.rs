use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::stdout;
use std::io::Cursor;
use std::path::{PathBuf, StripPrefixError};
use std::time::Duration;

use iroh::protocol::Router;
use iroh_blobs::rpc::client::blobs::{MemClient, WrapOption};
use iroh_blobs::util::SetTagOption;
use log::{debug, warn};
use niku_core::{ObjectEntry, ObjectKeepAliveRequest, ObjectKind, RegisteredObjectData};
use reqwest::Client;
use thiserror::Error;
use tokio::time;
use walkdir::WalkDir;
use zip::result::ZipError;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::format_bytes_to_string;

#[cfg(debug_assertions)]
const SEND_BACKEND_URL: &str = "http://localhost:4000";

#[cfg(not(debug_assertions))]
const SEND_BACKEND_URL: &str = "https://eu1.backend.niku.app";

#[derive(Error, Debug)]
pub enum SendError {
    #[error("Canonicalize the file path failed: {0}")]
    CanonicalizePathFailed(#[source] io::Error),

    #[error("An Iroh error has occurred: {0}")]
    IrohError(#[from] anyhow::Error),

    #[error("Unable to send the object to the backend server: {0}")]
    SendFileObjectFailed(#[source] reqwest::Error),

    #[error("Unable keep alive the object on the server: {0}")]
    KeepAliveFailed(#[source] reqwest::Error),

    #[error("The given path is not a file or a folder")]
    InvalidFileKind,

    #[error("The given file or folder doesn't have a Unicode filename")]
    NotUnicodeFilename,

    #[error("Trying to compress the folder failed: {0}")]
    ZipError(#[from] ZipError),

    #[error("Unable to do a IO operation over the in memory ZIP file: {0}")]
    ZipIoError(#[source] io::Error),

    #[error("Unable to open one of the files of the selected folder: {0}")]
    OpenFileFromFolderFailed(#[source] io::Error),

    #[error("Unable to strip the prefix from the given directory: {0}")]
    StripPrefixError(#[from] StripPrefixError),

    #[error("Unable to walk the folder: {0}")]
    WalkError(#[from] walkdir::Error),

    #[error("Unable to get the size of the terminal")]
    UnableToGetTheSizeOfTheTerminal(),
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
    println!("Packing file...");

    let blob = blobs_client
        .add_from_path(path.clone(), true, SetTagOption::Auto, WrapOption::NoWrap)
        .await?
        .finish()
        .await?;

    #[allow(clippy::expect_used)]
    let file_name = path
        .file_name()
        .expect("The path is always for a real file")
        .to_str()
        .ok_or(SendError::NotUnicodeFilename)?
        .to_string();

    Ok(ObjectEntry {
        node_address: router.endpoint().node_addr().await?,
        file_hash: blob.hash,
        kind: ObjectKind::File { name: file_name },
        size: blob.size,
    })
}

/// Creates a new object entry for a folder.
///
/// # Safety
/// Doesn't check if the given path is for a folder.
async unsafe fn create_folder_object_entry(
    directory_to_compress_path: &PathBuf,
    blobs_client: &MemClient,
    router: &Router,
) -> Result<ObjectEntry, SendError> {
    let mut data = Vec::new();
    let directory_to_compress_subpaths: Vec<Result<walkdir::DirEntry, walkdir::Error>> =
        WalkDir::new(directory_to_compress_path)
            .into_iter()
            .collect();

    let directory_to_compress_subpaths_length = directory_to_compress_subpaths.len();

    #[allow(clippy::expect_used)]
    let dir_name = directory_to_compress_path
        .clone()
        .file_name()
        .expect("The path is always for a real file")
        .to_str()
        .ok_or(SendError::NotUnicodeFilename)?
        .to_string();

    println!("Compressing the folder, please wait...");

    let terminal_width = term_size::dimensions()
        .ok_or(SendError::UnableToGetTheSizeOfTheTerminal())?
        .0;

    // The mess needed to compress a folder
    {
        let blob_buffer = Cursor::new(&mut data);
        let mut zip = ZipWriter::new(blob_buffer);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);

        let mut buffer = Vec::new();

        // Done like this because `.enumerate()` have some issues.
        let mut index = 0;
        for entry in directory_to_compress_subpaths {
            let entry = entry?;

            let path = entry.path();
            let name = path.strip_prefix(directory_to_compress_path)?;
            let path_as_string = name
                .to_str()
                .ok_or(SendError::NotUnicodeFilename)?
                .to_owned();

            index += 1;

            let status = format!(
                "  Item {index}/{} ({})",
                directory_to_compress_subpaths_length,
                path.file_name()
                    .ok_or(SendError::NotUnicodeFilename)?
                    .to_str()
                    .ok_or(SendError::NotUnicodeFilename)?
            );

            print!("{: <1$}\r", status, terminal_width);

            if name.as_os_str().is_empty() {
                continue;
            }

            if path.is_file() {
                zip.start_file(path_as_string, options)?;
                let mut file = File::open(path).map_err(SendError::OpenFileFromFolderFailed)?;

                file.read_to_end(&mut buffer)
                    .map_err(SendError::ZipIoError)?;

                zip.write_all(&buffer).map_err(SendError::ZipIoError)?;
                buffer.clear();

                continue;
            }

            if path.is_dir() {
                zip.add_directory(path_as_string, options)?;

                continue;
            }

            println!();
            warn!("Skipping file: {path_as_string}, it's neither a plain file or a directory!");
        }

        zip.finish()?;
    }

    let blob = blobs_client.add_bytes(data).await?;

    println!();
    println!();

    Ok(ObjectEntry {
        node_address: router.endpoint().node_addr().await?,
        file_hash: blob.hash,
        kind: ObjectKind::Folder { name: dir_name },
        size: blob.size,
    })
}

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
        } else if path.is_dir() {
            unsafe { create_folder_object_entry(&path, blobs_client, router).await }
        } else {
            Err(SendError::InvalidFileKind)
        }
    }?;

    debug!("Uploading object: {object:?}");

    let registration_data = client
        .put(format!("{SEND_BACKEND_URL}/objects"))
        .json(&object)
        .send()
        .await
        .map_err(SendError::SendFileObjectFailed)?
        .json::<RegisteredObjectData>()
        .await
        .map_err(SendError::SendFileObjectFailed)?;

    let object_id_with_whitespace = registration_data.id.replace("-", " ");

    match object.kind.clone() {
        ObjectKind::File { name } | ObjectKind::Folder { name } => {
            println!(
                "  Sending the {} '{}' ({})",
                object.kind,
                name,
                format_bytes_to_string(object.size)
            );
        }
    }
    println!(
        "  Your ID is: '{}' ({})",
        object_id_with_whitespace, registration_data.id
    );
    println!();
    println!("  On the other device, please run:");
    println!("  niku receive {}", registration_data.id);
    println!();
    println!("  Or use one of the official GUI apps:");
    println!("  https://niku.app/download");

    let mut interval = time::interval(Duration::from_secs(1));

    loop {
        debug!("Keeping alive the connection!");

        interval.tick().await;
        client
            .post(format!(
                "{SEND_BACKEND_URL}/objects/{}/keep-alive",
                registration_data.id
            ))
            .json(&ObjectKeepAliveRequest {
                keep_alive_key: registration_data.keep_alive_key.clone(),
            })
            .send()
            .await
            .map_err(SendError::KeepAliveFailed)?;
    }
}
