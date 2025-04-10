// Copyright 2025 Google LLC
// SPDX-License-Identifier: MPL

//! Backend in charge of making discovery possible on NIKU.

use tracing::error;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        // Config the logging with env vars and set the default level to "info"
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME")).into()),
        )
        .with_target(false)
        .compact()
        .init();

    match niku_backend::run().await {
        Ok(_) => (),
        Err(err) => error!("{err}"),
    }
}
