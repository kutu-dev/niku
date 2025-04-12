// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use std::path::PathBuf;

use anyhow::Result;
use log::{debug, info};
use niku::object::ObjectKind;
use niku::peer::Peer;
use tokio::try_join;

use super::{Cli, CliError};

impl Cli {
    pub(super) async fn receive(
        id: &str,
        output: &Option<PathBuf>,
        should_ask: bool,
    ) -> Result<(), CliError> {
        let id = id.replace("_", "-");

        let output = match output {
            Some(output) => Some(std::path::absolute(output)?),
            _ => None,
        };

        let peer = Peer::new().await?;

        let object_entry = peer.retrieve_object_entry(&id).await?;

        if should_ask {
            eprint!(
                "Download {} '{}' ({})? (Y/n): ",
                object_entry.kind,
                object_entry.name,
                niku::format_bytes_with_unit(object_entry.size)
            );

            let answer: String = text_io::read!("{}\n");
            let answer = answer.to_lowercase();

            if !["y", "yes", ""].contains(&answer.as_str()) {
                info!("Download canceled!");
                return Ok(());
            }
        } else {
            info!(
                "Downloading {} '{}' ({})",
                object_entry.kind,
                object_entry.name,
                niku::format_bytes_with_unit(object_entry.size)
            )
        }

        let (task, token) = crate::cli::generic_wait("Downloading object").await;

        peer.download_object_entry(&object_entry).await?;

        token.cancel();
        try_join!(task)?;

        let (task, token) = crate::cli::generic_wait("Exporting object").await;

        let (output_path, file_to_be_deleted_path) = match &object_entry.kind {
            ObjectKind::File => unsafe {
                (
                    peer.export_file_object_entry(&object_entry, &output)
                        .await?,
                    None,
                )
            },

            ObjectKind::Folder => unsafe {
                peer.export_folder_object_entry(&object_entry, &output)
                    .await?
            },
        };

        token.cancel();
        try_join!(task)?;

        info!(
            "Done! Object '{}' downloaded at '{}'",
            object_entry.name,
            output_path
                .into_os_string()
                .into_string()
                .map_err(|_| CliError::IntoStringError)?
        );

        if let Some(file_to_be_deleted_path) = file_to_be_deleted_path {
            debug!("Removing temporal file...");
            tokio::fs::remove_file(file_to_be_deleted_path).await?;
        }

        Ok(())
    }
}
