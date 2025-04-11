// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

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
