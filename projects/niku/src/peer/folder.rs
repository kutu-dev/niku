// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use chrono::{DateTime, Utc};
use iroh_blobs::rpc::client::blobs::WrapOption;
use iroh_blobs::store::{ExportFormat, ExportMode};
use iroh_blobs::util::SetTagOption;
use tokio::fs;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

use super::{Peer, PeerError};
use crate::object::{HashWrapper, NodeAddrWrapper, ObjectEntry, ObjectKind};

impl Peer {
    async fn create_temporal_zip_file(subfolder_name: &str) -> Result<PathBuf, PeerError> {
        let now: DateTime<Utc> = SystemTime::now().into();

        #[allow(clippy::expect_used)]
        let mut temporal_zip_path = crate::get_cache_path();

        temporal_zip_path.push(format!("{subfolder_name}/{}.zip", now.format("%+")));

        fs::create_dir_all(temporal_zip_path.parent().ok_or(PeerError::FolderIsRoot)?).await?;

        Ok(temporal_zip_path)
    }

    fn compress_a_directory(src_path: &Path, zip_file: &File) -> Result<(), PeerError> {
        let walkdir = WalkDir::new(src_path)
            .into_iter()
            .filter_map(|path| path.ok());

        let mut zip = zip::ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);

        let mut buffer = Vec::new();
        for entry in walkdir {
            let path = entry.path();
            let name = path.strip_prefix(src_path)?;
            let path_as_string = name
                .to_str()
                .map(str::to_owned)
                .ok_or(PeerError::NotUnicodePath)?;

            // Write file or directory explicitly
            // Some unzip tools unzip files with directory paths correctly, some do not!
            if path.is_file() {
                zip.start_file(path_as_string, options)?;
                let mut file = File::open(path)?;

                file.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
                buffer.clear();
            } else if !name.as_os_str().is_empty() {
                // Only if not root! Avoids path spec / warning
                // and mapname conversion failed error on unzip
                //
                zip.add_directory(path_as_string, options)?;
            }
        }

        zip.finish()?;

        Ok(())
    }

    fn decompress_a_directory(
        zip_file_path: &Path,
        destination_path: &Path,
    ) -> Result<(), PeerError> {
        let file = std::fs::File::open(zip_file_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            #[allow(clippy::expect_used)]
            let mut file = archive
                .by_index(i)
                .expect("The file should always have an index");

            let destination_path = destination_path.join(match file.enclosed_name() {
                Some(path) => path,
                None => continue,
            });

            if file.is_dir() {
                std::fs::create_dir_all(&destination_path)?;
            } else {
                if let Some(p) = destination_path.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = std::fs::File::create(&destination_path)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }

    /// Creates a new object entry for a file.
    ///
    /// # Safety
    /// Doesn't check if the given path is for a folder.
    pub async unsafe fn create_folder_object_entry(
        &mut self,
        src_path: PathBuf,
    ) -> Result<(ObjectEntry, PathBuf), PeerError> {
        let temporal_zip_path =
            Peer::create_temporal_zip_file("published-compressed-folders").await?;

        let temporal_zip_file = File::create(temporal_zip_path.clone())?;

        Peer::compress_a_directory(&src_path, &temporal_zip_file)?;

        let blob = self
            .blobs
            .client()
            .add_from_path(
                temporal_zip_path.clone(),
                true,
                SetTagOption::Auto,
                WrapOption::NoWrap,
            )
            .await?
            .finish()
            .await?;

        #[allow(clippy::expect_used)]
        let file_name = src_path
            .file_name()
            .expect("The path is always for a real file")
            .to_str()
            .ok_or(PeerError::NotUnicodePath)?
            .to_string();

        Ok((
            ObjectEntry {
                node_address: NodeAddrWrapper(self.router.endpoint().node_addr().await?),
                file_hash: HashWrapper(blob.hash),
                kind: ObjectKind::Folder,
                name: file_name,
                size: blob.size,
            },
            temporal_zip_path,
        ))
    }

    /// Export a previously downloaded folder object entry.
    ///
    /// # Safety
    /// Doesn't check neither if the given object is for a folder
    /// or if the object has been downloaded beforehand into the Iroh store.
    pub async unsafe fn export_folder_object_entry(
        &self,
        object_entry: &ObjectEntry,
        custom_output_path: &Option<PathBuf>,
    ) -> Result<(PathBuf, Option<PathBuf>), PeerError> {
        let output_path = if let Some(custom_output_path) = custom_output_path {
            custom_output_path.clone()
        } else {
            let mut cwd_path = std::env::current_dir()?;
            cwd_path.push(&object_entry.name);

            cwd_path
        };

        let temporal_zip_path =
            Peer::create_temporal_zip_file("downloaded-compressed-folders").await?;

        self.blobs
            .client()
            .export(
                object_entry.file_hash.0,
                temporal_zip_path.to_owned(),
                ExportFormat::Blob,
                ExportMode::Copy,
            )
            .await?
            .finish()
            .await?;

        Peer::decompress_a_directory(&temporal_zip_path, &output_path)?;

        Ok((output_path.to_owned(), Some(temporal_zip_path)))
    }
}
