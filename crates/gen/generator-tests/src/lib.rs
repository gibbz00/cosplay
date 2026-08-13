#![allow(missing_docs)]

include!(concat!(env!("OUT_DIR"), "/combined.rs"));

#[cfg(test)]
mod tests {
    use async_wayland_codec::Message;

    #[test]
    fn rename() {
        use super::rename::*;
        // NB: Pixelformat would normally be called Format without the rename,
        // which would then cause a compiler error.
        let _ = Format { format: PixelFormat::A };
    }

    #[test]
    fn opcodes() {
        use super::opcodes::*;

        assert_eq!(0, ReqA::OP_CODE);
        assert_eq!(1, ReqB::OP_CODE);
        assert_eq!(0, EvC::OP_CODE);
    }
}
