use std::num::NonZeroU32;

use serde::Deserialize;

/// Newtype wrapper around a non-zero u32.
///
/// See [versioning] for more info on how it is used.
///
/// [versioning]: https://wayland.freedesktop.org/docs/book/Protocol.html#versioning
#[derive(Debug, PartialEq, Deserialize)]
pub struct Version(pub(crate) NonZeroU32);

impl AsRef<NonZeroU32> for Version {
    fn as_ref(&self) -> &NonZeroU32 {
        &self.0
    }
}

impl Default for Version {
    fn default() -> Self {
        // SAFETY: 1 is not zero.
        let inner = unsafe { NonZeroU32::new_unchecked(1) };
        Self(inner)
    }
}
