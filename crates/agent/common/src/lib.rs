// TEMP:
#![allow(missing_docs)]

//! # `cosplay-agent` - Common utilities shared between `cosplay-client` and `cosplay-server`.

mod agent_marker;
pub use agent_marker::{Agent, Client, Server};

pub mod object_id_pool;

pub mod misc {
    use cosplay_codec::*;
    use cosplay_protocols_wayland::wl_display::WlDisplay;

    pub const WL_DISPLAY_ID: ObjectId<WlDisplay> = ObjectId::new(OpaqueObjectId::new(1));
}

pub mod geometry {
    //! Geometric primitives commonly found in protocol messages.

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    #[allow(missing_docs)]
    pub struct Rectangle {
        pub x: i32,
        pub y: i32,
        pub width: u16,
        pub height: u16,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Dimensions {
        pub width: u16,
        pub height: u16,
    }

    impl Dimensions {
        /// Helper for constructing `Self` from i32s.
        ///
        /// `i32` is the integer type commonly used for messages arguments sent over the wire.
        pub fn new_checked(width: i32, height: i32) -> Option<Self> {
            let width = u16::try_from(width).ok()?;
            let height = u16::try_from(height).ok()?;

            Some(Self { width, height })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn dimension_new_checked_ok() {
            let actual = Dimensions::new_checked(123, 456).unwrap();
            let expected = Dimensions { width: 123, height: 456 };
            assert_eq!(expected, actual);
        }

        #[test]
        fn dimension_new_checked_err() {
            assert!(Dimensions::new_checked(-1, 1).is_none());
            assert!(Dimensions::new_checked(1, -1).is_none());
            assert!(Dimensions::new_checked(1 << 18, 1).is_none());
            assert!(Dimensions::new_checked(1, 1 << 18).is_none());
        }
    }
}

// TODO(refactor): move to a separate crate?
pub mod pixel_format;
