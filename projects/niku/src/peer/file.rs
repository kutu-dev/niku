// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use std::path::PathBuf;

use anyhow::Result;
use iroh_blobs::rpc::client::blobs::WrapOption;
use iroh_blobs::store::{ExportFormat, ExportMode};
use iroh_blobs::util::SetTagOption;

use super::{Peer, PeerError};
use crate::object::{HashWrapper, NodeAddrWrapper, ObjectEntry, ObjectKind};

impl Peer {
    /// Creates a new object entry for a file.
    ///
    /// # Safety
    /// Doesn't check if the given path is for a file.
    pub async unsafe fn create_file_object_entry(
        &mut self,
        path: PathBuf,
    ) -> Result<ObjectEntry, PeerError> {
        let blob = self
            .blobs
            .client()
            .add_from_path(path.clone(), true, SetTagOption::Auto, WrapOption::NoWrap)
            .await?
            .finish()
            .await?;

        #[allow(clippy::expect_used)]
        let file_name = path
            .file_name()
            .expect("The path is always for a real file")
            .to_str()
            .ok_or(PeerError::NotUnicodePath)?
            .to_string();

        Ok(ObjectEntry {
            node_address: NodeAddrWrapper(self.router.endpoint().node_addr().await?),
            file_hash: HashWrapper(blob.hash),
            kind: ObjectKind::File,
            name: file_name,
            size: blob.size,
        })
    }

    /// Export a previously downloaded file object entry.
    ///
    /// # Safety
    /// Doesn't check neither if the given object is for a file
    /// or if the object has been downloaded beforehand into the Iroh store.
    pub async unsafe fn export_file_object_entry(
        &self,
        object_entry: &ObjectEntry,
        custom_output_path: &Option<PathBuf>,
    ) -> Result<PathBuf, PeerError> {
        let output_path = if let Some(custom_output_path) = custom_output_path {
            custom_output_path.clone()
        } else {
            let mut cwd_path = std::env::current_dir()?;
            cwd_path.push(&object_entry.name);

            cwd_path
        };

        self.blobs
            .client()
            .export(
                object_entry.file_hash.0,
                output_path.to_owned(),
                ExportFormat::Blob,
                ExportMode::Copy,
            )
            .await?
            .finish()
            .await?;

        Ok(output_path.to_owned())
    }
}
