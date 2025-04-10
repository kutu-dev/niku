// Copyright 2025 Google LLC
// SPDX-License-Identifier: MPL

//! Shared code for all the crates of the NIKU project.

use std::fmt::{Debug, Display};
use std::io::Write;

use env_logger::fmt::style::{AnsiColor, Style};
use env_logger::Env;
use iroh::NodeAddr;
use iroh_blobs::Hash;
pub use log;
use log::Level;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The kind of an object
pub enum ObjectKind {
    /// The object is a file
    File {
        /// The name of the file
        name: String,
    },

    /// The object is a folder
    Folder {
        /// The name of the folder
        name: String,
    },
}

impl Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            ObjectKind::File { name: _ } => "file",
            ObjectKind::Folder { name: _ } => "folder",
        };

        write!(f, "{}", text)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Entry that holds the relevant data of a object available for downloading.
pub struct ObjectEntry {
    /// The [iroh] address of the node that is hosting the file.
    pub node_address: NodeAddr,

    /// The file hash used by [iroh_blobs] protocol to access the file.
    pub file_hash: Hash,

    /// The kind of object to be download.
    pub kind: ObjectKind,

    /// The number of bytes of the object.
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Relevant metadata about the state of the uploaded object on the backend server.
pub struct RegisteredObjectData {
    /// The ID of the object.
    pub id: String,

    /// A private UUIDv4 that must be used on a [ObjectKeepAliveRequest] to avoid the backend server deleting the object entry.
    pub keep_alive_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Request that can be send to the backend server the avoid it deleting the object entry.
pub struct ObjectKeepAliveRequest {
    /// The private UUIDv4 that has been returned by [RegisteredObjectData] used to identify and authenticate the object refresh.
    pub keep_alive_key: String,
}

/// Set a useful default configuration for CLI logging with [env_logger].
pub fn set_cli_logging() {
    // Set the minimum log level to `warn`
    // TRACK: https://github.com/rust-cli/env_logger/issues/162
    env_logger::Builder::from_env(Env::default().default_filter_or("warn"))
        .format(move |buf, record| {
            let bold_red_style = Style::new().bold().fg_color(Some(AnsiColor::Red.into()));
            let bold_cyan_style = Style::new().bold().fg_color(Some(AnsiColor::Cyan.into()));
            let bold_green_style = Style::new().bold().fg_color(Some(AnsiColor::Blue.into()));
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
