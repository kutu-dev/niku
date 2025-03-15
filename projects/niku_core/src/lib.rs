//! Share code for all the crates on the NIKU project.

use env_logger::fmt::style::{AnsiColor, Style};
use env_logger::Env;
use iroh::NodeAddr;
use iroh_blobs::Hash;
use log::Level;
use serde::{Deserialize, Serialize};
use std::io::Write;

pub use log;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UploadTicket {
    pub node_addr: NodeAddr,
    pub file_hash: Hash,
}

/// Set a useful default configuration for logging with [env_logger].
pub fn set_logging() {
    // Set the minimum log level to `info`
    // TRACK: https://github.com/rust-cli/env_logger/issues/162
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let bold_red_style = Style::new().bold().fg_color(Some(AnsiColor::Red.into()));
            let bold_cyan_style = Style::new().bold().fg_color(Some(AnsiColor::Cyan.into()));
            let bold_green_style = Style::new().bold().fg_color(Some(AnsiColor::Green.into()));
            let bold_yellow_style = Style::new().bold().fg_color(Some(AnsiColor::Yellow.into()));
            let bold_magenta_style = Style::new()
                .bold()
                .fg_color(Some(AnsiColor::Magenta.into()));

            let header = match record.level() {
                Level::Trace => format!("[ {bold_magenta_style}TRACE{bold_magenta_style:#} ]"),
                Level::Debug => format!("[ {bold_cyan_style}DEBUG{bold_cyan_style:#} ]"),
                Level::Info => format!("[ {bold_green_style}INFO{bold_green_style:#} ]"),
                Level::Warn => format!("[ {bold_yellow_style}WARN{bold_yellow_style:#} ]"),
                Level::Error => format!("[ {bold_red_style}ERROR{bold_red_style:#} ]"),
            };

            writeln!(buf, "{header} {}", record.args())
        })
        .init();
}
