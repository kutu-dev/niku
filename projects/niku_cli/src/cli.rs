// Copyright 2025 Google LLC
// SPDX-License-Identifier: MPL

//! Main internal library of the NIKU command line app.

use std::ffi::IntoStringError;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use log::error;
use niku::peer::PeerError;
use thiserror::Error;
use tokio::task::{JoinError, JoinHandle};
use tokio_util::sync::CancellationToken;

mod prune;
mod receive;
mod send;

#[derive(Parser)]
#[command(name = "NIKU")]
#[command(about, long_about = None)]
/// NIKU: Send files fast and privately with the power of P2P technologies
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Prune the cache
    Prune,

    /// Send an object.
    Send { path: PathBuf },

    /// Receive an object.
    Receive {
        id: String,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Error, Debug)]
/// Errors that may happen when running the app.
pub enum CliError {
    /// An error that may happen when interacting with a peer.
    #[error("An error has occured while interacting with the peer: {0}")]
    PeerError(#[from] PeerError),

    /// The given path is invalid.
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// The given path is invalid.
    #[error("IO error: {0}")]
    FsExtraError(#[from] fs_extra::error::Error),

    /// The given path is not for a file or for a folder.
    #[error("The given path is not for a file or for a folder")]
    ThePathIsNotAFileOrAFolder,

    #[error("Unable to join a task: {0}")]
    JoinTaskFailed(#[from] JoinError),

    #[error("The path where the file was downloaded is not UTF-8 (Unicode) encoded")]
    IntoStringError,
}

impl Cli {
    /// Run the CLI correct subcommand.
    pub async fn run(&self) -> Result<(), CliError> {
        match &self.command {
            Commands::Prune => Cli::prune().await?,
            Commands::Send { path } => Cli::send(path).await?,
            Commands::Receive { id, output } => Cli::receive(id, output).await?,
        }

        Ok(())
    }
}

async fn generic_wait(message: &str) -> (JoinHandle<()>, CancellationToken) {
    eprint!("{message}: .");

    let token = CancellationToken::new();
    let cloned_token = token.clone();

    let task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(20));

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

    (task, token)
}
