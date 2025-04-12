use std::path::Path;
use std::time::Duration;
use std::fs;

use anyhow::Result;
use console::Emoji;
use log::{debug, info};
use niku::object::ObjectKind;
use niku::peer::Peer;
use tokio_util::sync::CancellationToken;

use super::{Cli, CliError};

impl Cli {
    pub(super) async fn send(path: &Path) -> Result<(), CliError> {
        let mut peer = Peer::new().await?;

        let path = fs::canonicalize(path)?;

        let (object_entry, file_to_be_deleted_path) = if path.is_file() {
            (unsafe { peer.create_file_object_entry(path).await? }, None)
        } else if path.is_dir() {
            eprint!("Compressing folder: ");

            let token = CancellationToken::new();
            let cloned_token = token.clone();

            let task = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(100));

                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            eprint!(".");
                        }

                        _ = cloned_token.cancelled() => {
                            eprintln!();
                            break;
                        }
                    }
                }
            });

            let (object_entry, file_to_be_deleted_path) =
                unsafe { peer.create_folder_object_entry(path).await? };

            task.abort();

            (object_entry, Some(file_to_be_deleted_path))
        } else {
            return Err(CliError::ThePathIsNotAFileOrAFolder);
        };

        let registered_object_entry = peer.publish_object_entry(&object_entry).await?;

        match object_entry.kind.clone() {
            ObjectKind::File { name } | ObjectKind::Folder { name } => {
                info!(
                    "{} Sending {} '{}'",
                    Emoji("📤 ", " "),
                    object_entry.kind,
                    name
                );
            }
        }

        let object_id_with_whitespaces = registered_object_entry.id.replace("-", " ");

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

        let mut interval = tokio::time::interval(Duration::from_secs(1));

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
