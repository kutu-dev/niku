// Copyright 2025 Google LLC
// SPDX-License-Identifier: MPL

use std::env::{self};
use std::io;
use std::path::{Path, PathBuf};

use iroh_blobs::rpc::client::blobs::MemClient;
use iroh_blobs::store::{ExportFormat, ExportMode};
use niku_core::{ObjectEntry, ObjectKind};
use reqwest::Client;
use thiserror::Error;

use crate::format_bytes_to_string;

#[derive(Error, Debug)]
pub enum ReceiveError {
    #[error("Unable to retrieve the object entry to the server: {0}")]
    BackendRequestFailed(#[source] reqwest::Error),

    #[error("The current working directory is not usable: {0}")]
    CurrentWorkingDirectoryInvalid(#[source] io::Error),

    #[error("Unable to open the cached compressed file: {0}")]
    UnableToOpenTheCachedFile(#[source] io::Error),

    #[error("Unable to handle the ZIP file: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("An unknown has occurred: {0}")]
    Unknown(#[from] anyhow::Error),
}

pub(crate) async fn receive(
    id: &str,
    client: Client,
    blobs_client: &MemClient,
    custom_path: Option<&String>,
) -> Result<(), ReceiveError> {
    let object_entry = client
        .get(format!("http://localhost:4000/objects/{id}"))
        .send()
        .await
        .map_err(ReceiveError::BackendRequestFailed)?;

    println!("{object_entry:?}");

    let object_entry = object_entry
        .json::<ObjectEntry>()
        .await
        .map_err(ReceiveError::BackendRequestFailed)?;

    let object_name = match object_entry.kind.clone() {
        ObjectKind::Folder { name } | ObjectKind::File { name } => name,
    };

    print!(
        "  Download {} '{object_name}' ({})? (Y/n): ",
        object_entry.kind,
        format_bytes_to_string(object_entry.size)
    );

    let answer: String = text_io::read!("{}\n");
    let answer = answer.to_lowercase();

    if !["y", "yes", ""].contains(&answer.as_str()) {
        println!("Download canceled!");
        return Ok(());
    }

    blobs_client
        .download(object_entry.file_hash, object_entry.node_address.clone())
        .await?
        .finish()
        .await?;

    let final_object_path = if let Some(path) = custom_path {
        PathBuf::from(path)
    } else {
        let mut path = env::current_dir().map_err(ReceiveError::CurrentWorkingDirectoryInvalid)?;

        path.push(object_name.clone());

        path
    };

    match object_entry.kind {
        ObjectKind::File { name: _ } => {
            blobs_client
                .export(
                    object_entry.file_hash,
                    final_object_path.clone(),
                    ExportFormat::Blob,
                    ExportMode::Copy,
                )
                .await?
                .finish()
                .await?;
        }

        ObjectKind::Folder { name: _ } => {
            #[allow(clippy::expect_used)]
            let mut tmp_zip_path = dirs::cache_dir()
                .expect("NIKU is not available on platforms without a cache storage");
            tmp_zip_path.push("app.niku/downloads");
            tmp_zip_path.push(format!("{}.zip", object_entry.file_hash.to_string()));

            blobs_client
                .export(
                    object_entry.file_hash,
                    tmp_zip_path.clone(),
                    ExportFormat::Blob,
                    ExportMode::Copy,
                )
                .await?
                .finish()
                .await?;

            let file = std::fs::File::open(tmp_zip_path)
                .map_err(ReceiveError::UnableToOpenTheCachedFile)?;
            let mut archive = zip::ZipArchive::new(file)?;

            for i in 0..archive.len() {
                #[allow(clippy::expect_used)]
                let mut file = archive
                    .by_index(i)
                    .expect("The file should always have an index");

                let outpath = match file.enclosed_name() {
                    Some(path) => path,
                    None => continue,
                };

                let outpath = Path::new(&final_object_path)
                    .join(&object_name)
                    .join(outpath);

                if file.is_dir() {
                    println!("File {} extracted to \"{}\"", i, outpath.display());
                    std::fs::create_dir_all(&outpath).unwrap();
                } else {
                    println!(
                        "File {} extracted to \"{}\" ({} bytes)",
                        i,
                        outpath.display(),
                        file.size()
                    );
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p).unwrap();
                        }
                    }
                    let mut outfile = std::fs::File::create(&outpath).unwrap();
                    io::copy(&mut file, &mut outfile).unwrap();
                }
            }
        }
    }

    Ok(())
}
