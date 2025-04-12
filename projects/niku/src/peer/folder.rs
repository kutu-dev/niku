use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use chrono::{DateTime, Utc};
use iroh_blobs::rpc::client::blobs::WrapOption;
use iroh_blobs::util::SetTagOption;
use tokio::fs;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

use super::{Peer, PeerError};
use crate::object::{HashWrapper, NodeAddrWrapper, ObjectEntry, ObjectKind};

impl Peer {
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
                let mut file =
                    File::open(path).map_err(PeerError::UnableToWritoIntoTheFilesystem)?;

                file.read_to_end(&mut buffer)
                    .map_err(PeerError::UnableToWritoIntoTheFilesystem)?;
                zip.write_all(&buffer)
                    .map_err(PeerError::UnableToWritoIntoTheFilesystem)?;
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

    /// Creates a new object entry for a file.
    ///
    /// # Safety
    /// Doesn't check if the given path is for a folder.
    pub async unsafe fn create_folder_object_entry(
        &mut self,
        src_path: PathBuf,
    ) -> Result<(ObjectEntry, PathBuf), PeerError> {
        let now: DateTime<Utc> = SystemTime::now().into();

        #[allow(clippy::expect_used)]
        let mut temporal_zip_path = crate::get_cache_path();

        temporal_zip_path.push(format!(
            "published-compressed-folders/{}.zip",
            now.format("%+")
        ));

        fs::create_dir_all(temporal_zip_path.parent().ok_or(PeerError::FolderIsRoot)?)
            .await
            .map_err(PeerError::UnableToWritoIntoTheFilesystem)?;

        let temporal_zip_file = File::create(temporal_zip_path.clone())
            .map_err(PeerError::UnableToWritoIntoTheFilesystem)?;

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
                kind: ObjectKind::Folder { name: file_name },
                size: blob.size,
            },
            temporal_zip_path,
        ))
    }
}
