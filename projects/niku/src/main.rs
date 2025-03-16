//! Main NIKU command line app.

use log::error;
use niku::run;
use niku_core::set_cli_logging;

#[tokio::main]
async fn main() {
    set_cli_logging();

    match run().await {
        Ok(_) => (),
        Err(err) => error!("{err}"),
    }
}
