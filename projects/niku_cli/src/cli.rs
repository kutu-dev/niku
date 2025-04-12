// Copyright 2025 Google LLC
// SPDX-License-Identifier: MPL

//! Main internal library of the NIKU command line app.

use std::path::PathBuf;
use std::io;

use clap::{Parser, Subcommand};
use log::error;
use niku::peer::PeerError;
use thiserror::Error;

mod prune;
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

    /// Send a file
    Send { path: PathBuf },
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
}

impl Cli {
    /// Run the CLI correct subcommand.
    pub async fn run(&self) -> Result<(), CliError> {
        match &self.command {
            Commands::Prune => Cli::prune().await?,
            Commands::Send { path } => Cli::send(path).await?,
        }

        Ok(())
    }
}
