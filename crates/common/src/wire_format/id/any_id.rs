use std::{marker::PhantomData, num::NonZeroU32, ops::RangeInclusive};

use crate::*;

/// Opaque Object ID
///
/// Used when it's not possible to know whether it is client or server originated.
#[derive(Debug, PartialEq)]
pub struct AnyObjectId(NonZeroU32);

impl AnyObjectId {
    /// Zero is used to represent a null or non-existent object, so `raw == 0` returns `Ok(None)`.
    pub(crate) const fn from_raw(raw: u32) -> Option<Self> {
        // IMPROVEMENT: use Option::map when or if const_result_trait_fn stabilizes
        match NonZeroU32::new(raw) {
            Some(inner) => Some(Self(inner)),
            None => None,
        }
    }

    pub(crate) const fn inner(&self) -> u32 {
        self.0.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_raw() {
        let actual = AnyObjectId::from_raw(0);
        assert!(actual.is_none());

        let actual = AnyObjectId::from_raw(1).unwrap();
        assert_eq!(1, actual.inner());
    }
}
