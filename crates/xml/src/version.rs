use std::num::NonZeroU32;

use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Version(pub(crate) NonZeroU32);

impl Default for Version {
    fn default() -> Self {
        // SAFETY: 1 is not zero.
        let inner = unsafe { NonZeroU32::new_unchecked(1) };
        Self(inner)
    }
}
