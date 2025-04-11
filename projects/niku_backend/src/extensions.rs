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
