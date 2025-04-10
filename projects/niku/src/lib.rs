// Copyright 2025 Google LLC
// SPDX-License-Identifier: MPL

//! Main NIKU command line app internal library.

mod receive;
mod run;
mod send;

pub use run::run;

const VERSION: &str = "0.0.1";

pub(crate) fn get_backend_url(prefix: &str) -> Option<String> {
    let url = match prefix {
        "test" => Some("http://localhost:4000"),
        "the" => Some("https://eu1.backend.niku.app"),
        _ => None,
    };

    url.map(String::from)
}

const BYTES_IN_A_KIBIBYTE: u64 = 1024;

pub(crate) fn format_bytes_to_string(size: u64) -> String {
    if size < BYTES_IN_A_KIBIBYTE {
        format!("{} B", size)
    } else {
        format!("{} KiB", size / BYTES_IN_A_KIBIBYTE)
    }
}
