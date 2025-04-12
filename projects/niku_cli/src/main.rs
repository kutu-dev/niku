// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Main NIKU cli client.

mod cli;

use std::io::Write;

use clap::builder::styling::{AnsiColor, Style};
use clap::Parser;
use env_logger::Env;
use log::{error, Level};

use crate::cli::Cli;

/// Set a useful default configuration for CLI logging with [env_logger].
pub fn set_cli_logging() {
    // Set the minimum log level to `warn`
    // TRACK: https://github.com/rust-cli/env_logger/issues/162
    env_logger::Builder::from_env(Env::default().default_filter_or("warn"))
        .format(move |buf, record| {
            let bold_red_style = Style::new().bold().fg_color(Some(AnsiColor::Red.into()));
            let bold_cyan_style = Style::new().bold().fg_color(Some(AnsiColor::Cyan.into()));
            let bold_yellow_style = Style::new().bold().fg_color(Some(AnsiColor::Yellow.into()));
            let bold_magenta_style = Style::new()
                .bold()
                .fg_color(Some(AnsiColor::Magenta.into()));

            let header = match record.level() {
                Level::Trace => format!("[ {bold_magenta_style}TRACE{bold_magenta_style:#} ] "),
                Level::Debug => format!("[ {bold_cyan_style}DEBUG{bold_cyan_style:#} ] "),
                Level::Info => String::from(""),
                Level::Warn => format!("[ {bold_yellow_style}WARN{bold_yellow_style:#} ] "),
                Level::Error => format!("[ {bold_red_style}ERROR{bold_red_style:#} ] "),
            };

            writeln!(buf, "{header}{}", record.args())
        })
        .init();
}

#[tokio::main]
async fn main() {
    set_cli_logging();

    let cli = Cli::parse();

    match cli.run().await {
        Ok(_) => (),
        Err(err) => error!("{err}"),
    }
}
