// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! The NIKU client library.

use log::debug;

uniffi::setup_scaffolding!();

pub mod backend;
pub mod object;
pub mod peer;
use std::path::PathBuf;
pub(crate) const CACHE_PREFIX: &str = "app.niku";

/// Get the system dependant user cache storage path.
pub fn get_cache_path() -> PathBuf {
    #[allow(clippy::expect_used)]
    let mut cache_path =
        dirs::cache_dir().expect("NIKU is not available on systems without cache dir");
    cache_path.push(CACHE_PREFIX);

    cache_path
}

#[uniffi::export]
/// Get the correct backend address given its prefix.
pub(crate) fn get_backend_address_from_prefix(prefix: &str) -> Option<String> {
    let url = match prefix {
        "test" => Some("http://localhost:8080"),
        "the" => Some("https://eu1.backend.niku.app"),
        _ => None,
    };

    url.map(String::from)
}

pub(crate) fn get_recommended_backend_address() -> String {
    String::from(if cfg!(debug_assertions) {
        debug!("Debug mode enabled, trying to use local backend...");
        "http://localhost:8080"
    } else {
        "https://eu1.backend.niku.app"
    })
}

const BYTES_IN_A_KIBIBYTE: u64 = 1024;

/// Format a quantity of bytes with a suffix from B (Byte) to GiB (Gibibyte)
pub fn format_bytes_with_unit(size: u64) -> String {
    let scale = size / BYTES_IN_A_KIBIBYTE;

    let (suffix, power) = match scale {
        0 => ("B", 0),

        1 => ("KiB", 1),

        _ => {
            let scale = size / BYTES_IN_A_KIBIBYTE.pow(3);

            if scale == 0 {
                ("MiB", 2)
            } else {
                ("GiB", 3)
            }
        }
    };

    format!(
        "{:.2} {suffix}",
        size as f32 / BYTES_IN_A_KIBIBYTE.pow(power) as f32
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_with_unit_byte() {
        assert_eq!(format_bytes_with_unit(876), "876.00 B")
    }

    #[test]
    fn test_format_bytes_with_unit_kibibyte() {
        assert_eq!(format_bytes_with_unit(1500), "1.46 KiB")
    }

    #[test]
    fn test_format_bytes_with_unit_mebibyte() {
        assert_eq!(format_bytes_with_unit(8_000_000), "7.63 MiB")
    }

    #[test]
    fn test_format_bytes_with_unit_gibibyte() {
        assert_eq!(format_bytes_with_unit(7_800_000_000), "7.26 GiB")
    }
}
