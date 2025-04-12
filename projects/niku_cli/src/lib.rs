// Copyright 2025 Google LLC
// SPDX-License-Identifier: MPL

//! Main internal library of the NIKU command line app.

use std::path::PathBuf;
use std::time::Duration;
use std::{fs, io};

use clap::builder::TypedValueParser;
use clap::{Parser, Subcommand};
use console::Emoji;
use log::{debug, error, info};
use niku::object::ObjectKind;
use niku::peer::{Peer, PeerError};
use thiserror::Error;
use tokio_util::sync::CancellationToken;

#[derive(Parser)]
#[command(name = "NIKU")]
#[command(about, long_about = None)]
/// NIKU: Send files fast and privately with the power of P2P technologies
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a file
    Send { path: PathBuf },
}

#[derive(Error, Debug)]
/// Errors that may happen when running the app.
pub enum RunError {
    /// An error that may happen when interacting with a peer.
    #[error("An error has occured while interacting with the peer: {0}")]
    PeerError(#[from] PeerError),

    /// The given path is invalid.
    #[error("The given path is invalid: {0}")]
    PathIsInvalid(#[from] io::Error),

    /// The given path is not for a file or for a folder.
    #[error("The given path is not for a file or for a folder")]
    ThePathIsNotAFileOrAFolder,
}

/// Run the app.
pub async fn run() -> Result<(), RunError> {
    let cli = Cli::parse();

    let mut peer = Peer::new().await?;

    match cli.command {
        Commands::Send { path } => {
            let path = fs::canonicalize(path)?;

            let object_entry = if path.is_file() {
                unsafe { peer.create_file_object_entry(path).await? }
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

                let object_entry = unsafe { peer.create_folder_object_entry(path).await? };

                task.abort();

                object_entry
            } else {
                return Err(RunError::ThePathIsNotAFileOrAFolder);
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

            debug!("Prunning the cache...");
            niku::prune_cache()?;
        }
    }

    debug!("Shuting down the peer...");
    peer.async_drop().await?;

    Ok(())
}
