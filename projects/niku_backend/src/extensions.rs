// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Useful extra functions for common types.

use rand::seq::IndexedRandom;

pub(crate) trait StringSliceExt {
    /// Get a random value from a `&[&str]`
    ///
    /// # Safety
    /// The given slice must not be empty.
    unsafe fn get_random(&self) -> String;
}

impl StringSliceExt for [String] {
    unsafe fn get_random(&self) -> String {
        #[allow(clippy::expect_used)]
        self.choose(&mut rand::rng())
            .expect("The vector should never be empty")
            .to_string()
    }
}
