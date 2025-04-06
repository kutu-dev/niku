//! Main NIKU command line app.

mod receive;
mod send;
mod string;

use clap::{arg, Command};
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_blobs::net_protocol::Blobs;
use log::error;
use receive::ReceiveError;
use reqwest::Client;
use send::SendError;
use thiserror::Error;

const VERSION: &str = "0.0.1";

#[cfg(debug_assertions)]
const BASE_BACKEND_URL: &str = "http://localhost:4000";

#[cfg(not(debug_assertions))]
const BASE_BACKEND_URL: &str = "https://eu1.backend.niku.app";

fn create_command() -> Command {
    Command::new("niku")
        .about("NIKU: Send files fast and privately with the power of P2P technologies")
        .version(VERSION)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("send")
                .about("Send a file")
                .arg(arg!(<PATH> "The path of the file to be sent"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("receive")
                .about("Send a file")
                .arg(arg!(<ID> "The ID of the file to be downloaded"))
                .arg_required_else_help(true)
                .arg(
                    arg!(-o --output <PATH> "Set a custom path and filename to the file to be downloaded")
                ),
        )
}

#[derive(Error, Debug)]
/// Errors that may happen when running the app.
pub enum RunError {
    /// Unable to send the file.
    #[error("Unable to send the file: {0}")]
    SendFailed(#[from] SendError),

    /// Unable to receive the file.
    #[error("Unable to receive the file: {0}")]
    RetrieveFailed(#[from] ReceiveError),

    /// An error from Iroh.
    #[error("An error from Iroh has been raised: {0}")]
    IrohError(#[from] anyhow::Error),
}

/// Run the app.
pub async fn run() -> Result<(), RunError> {
    let client = Client::new();
    let endpoint = Endpoint::builder().bind().await?;

    let blobs_protocol = Blobs::memory().build(&endpoint);
    let blobs_client = blobs_protocol.client();

    let router = Router::builder(endpoint)
        .accept(iroh_blobs::ALPN, blobs_protocol.clone())
        .spawn()
        .await?;

    match create_command().get_matches().subcommand() {
        Some(("send", sub_matches)) =>
        {
            #[allow(clippy::expect_used)]
            send::send(
                sub_matches
                    .get_one::<String>("PATH")
                    .expect("This is a required argument"),
                client.clone(),
                blobs_client,
                &router,
            )
            .await?
        }

        Some(("receive", sub_matches)) => {
            receive::receive(
                #[allow(clippy::expect_used)]
                sub_matches
                    .get_one::<String>("ID")
                    .expect("This is a required argument"),
                client.clone(),
                blobs_client,
            )
            .await?
        }

        _ => unreachable!(),
    };

    router.shutdown().await?;

    Ok(())
}
