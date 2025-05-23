// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use console::Emoji;
use log::{debug, info};
use niku::peer::Peer;
use tokio::try_join;

use super::{Cli, CliError};

#[cfg(debug_assertions)]
const KEEP_ALIVE_OBJECT_SECONDS: u64 = 2;

#[cfg(not(debug_assertions))]
const KEEP_ALIVE_OBJECT_SECONDS: u64 = 2 * 60;

impl Cli {
    pub(super) async fn send(path: &Path) -> Result<(), CliError> {
        let mut peer = Peer::new().await?;

        let path = fs::canonicalize(path)?;

        let (object_entry, file_to_be_deleted_path) = if path.is_file() {
            (unsafe { peer.create_file_object_entry(path).await? }, None)
        } else if path.is_dir() {
            let (task, token) = crate::cli::generic_wait("Compressing folder").await;

            let (object_entry, file_to_be_deleted_path) =
                unsafe { peer.create_folder_object_entry(path).await? };

            token.cancel();
            try_join!(task)?;

            (object_entry, Some(file_to_be_deleted_path))
        } else {
            return Err(CliError::ThePathIsNotAFileOrAFolder);
        };

        let registered_object_entry = peer.publish_object_entry(&object_entry).await?;

        let object_id_with_whitespaces = registered_object_entry.id.replace("-", " ");

        info!(
            "{} Sending {} '{}'",
            Emoji("📤 ", " "),
            object_entry.kind,
            object_entry.name
        );
        info!(
            " Your ID is: '{}' ({})",
            object_id_with_whitespaces, registered_object_entry.id
        );
        info!("");
        info!("{} On the other device, please run:", Emoji("📥", " "));
        info!("  niku receive {}", registered_object_entry.id);
        info!("");
        info!("{} Or use one of the official GUI apps:", Emoji("🌐", " "));
        info!("  https://niku.app/download");

        let mut interval = tokio::time::interval(Duration::from_secs(KEEP_ALIVE_OBJECT_SECONDS));

        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    break;
                }

                _ = interval.tick() => {
                    debug!("Keeping alive the object...");
                    peer.keep_alive_object_entry(&registered_object_entry).await?;
                }
            }
        }

        if let Some(file_to_be_deleted_path) = file_to_be_deleted_path {
            debug!("Removing temporal file...");
            tokio::fs::remove_file(file_to_be_deleted_path).await?;
        }

        Ok(())
    }
}
