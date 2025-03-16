//! Main NIKU command line app.

mod receive;
mod send;

use clap::{arg, Command};
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_blobs::net_protocol::Blobs;
use log::error;
use niku_core::set_logging;
use receive::ReceiveError;
use reqwest::Client;
use send::SendError;
use thiserror::Error;

const VERSION: &str = "0.0.1";
const BASE_BACKEND_URL: &str = "http://localhost:4817";

fn get_command() -> Command {
    Command::new("niku")
        .about("NIKU: Send files fast and privately with P2P")
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
enum RunError {
    #[error("Unable to send the file: {0}")]
    SendFailed(#[from] SendError),

    #[error("Unable to receive the file: {0}")]
    RetrieveFailed(#[from] ReceiveError),

    #[error("An unknown has occurred: {0}")]
    Unknown(#[from] anyhow::Error),
}

async fn run() -> Result<(), RunError> {
    set_logging(false);

    let client = Client::new();

    let endpoint = Endpoint::builder().discovery_n0().bind().await?;

    let blobs_protocol = Blobs::memory().build(&endpoint);
    let blobs_client = blobs_protocol.client();

    let router = Router::builder(endpoint)
        .accept(iroh_blobs::ALPN, blobs_protocol.clone())
        .spawn()
        .await?;

    match get_command().get_matches().subcommand() {
        Some(("send", sub_matches)) => {
            send::send(
                sub_matches
                    .get_one::<String>("PATH")
                    .expect("Required argument"),
                client.clone(),
                blobs_client,
                &router,
            )
            .await?
        }

        Some(("receive", sub_matches)) => {
            receive::receive(
                sub_matches
                    .get_one::<String>("ID")
                    .expect("Required argument"),
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

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => (),
        Err(err) => error!("{err}"),
    }
}
