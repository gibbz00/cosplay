#![allow(missing_docs)]

include!(concat!(env!("OUT_DIR"), "/combined.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename() {
        let _ = rename::Format { format: rename::PixelFormat::A };
    }
}
