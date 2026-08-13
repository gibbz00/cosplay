#![allow(missing_docs)]

include!(concat!(env!("OUT_DIR"), "/combined.rs"));

#[cfg(test)]
mod tests {
    use async_wayland_codec::{Message, ObjectId, OpaqueObjectId, WaylandMemoryBuffer};

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

    #[tokio::test]
    async fn encoding() {
        use super::wl_encoding::*;

        let object_id = ObjectId::<WlEncoding>::new(OpaqueObjectId(1));
        let text = "Some string 🦀".to_string();

        let message = BasicMessage { text: text.clone() };

        let mut sink = async_wayland_codec::WaylandMessageSink::new(WaylandMemoryBuffer::default());
        sink.send_concrete(object_id, message).await.unwrap();

        let mut stream = async_wayland_codec::WaylandMessageStream::new(sink.into_inner());
        let (received_id, received_message) = stream.receive_concrete::<BasicMessage>().await.unwrap().unwrap();

        assert_eq!(received_id, object_id);
        assert_eq!(text, received_message.text);
    }

    #[tokio::test]
    async fn enumeration() {
        use super::{wl_enum::*, wl_other::*};

        let object_id = ObjectId::<WlEnum>::new(OpaqueObjectId(1));
        let local = Local::_1Y;
        let remote = Remote::B;

        let message = EnumMessage { local, remote };

        let mut sink = async_wayland_codec::WaylandMessageSink::new(WaylandMemoryBuffer::default());
        sink.send_concrete(object_id, message).await.unwrap();

        let mut stream = async_wayland_codec::WaylandMessageStream::new(sink.into_inner());
        let (_, received_message) = stream.receive_concrete::<EnumMessage>().await.unwrap().unwrap();

        assert_eq!(local, received_message.local);
        assert_eq!(remote, received_message.remote);
    }
}
