// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! The NIKU client library.

pub mod backend;
pub mod object;

pub(crate) fn get_backend_url(prefix: &str) -> Option<String> {
    let url = match prefix {
        "test" => Some("http://localhost:8080"),
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
