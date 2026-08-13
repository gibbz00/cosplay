#![allow(missing_docs)]

include!(concat!(env!("OUT_DIR"), "/combined.rs"));

#[cfg(test)]
mod tests {
    use cosplay_codec::{DecodeMessage, EncodeMessage, Message, ObjectId, OpaqueObjectId, WaylandMemoryBuffer};

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
    async fn primitive_encoding() {
        use super::wl_encoding::*;

        let text = "Some string 🦀".to_string();

        let received_message = roundtrip_message(BasicMessage { text: text.clone() }).await;

        assert_eq!(text, received_message.text);
    }

    #[tokio::test]
    async fn object_encoding() {
        use super::{wl_a::*, wl_b::WlB};

        let id = ObjectId::<WlB>::new(OpaqueObjectId(2));

        let received_message = roundtrip_message(Request { id }).await;

        assert_eq!(id, received_message.id);
    }

    #[tokio::test]
    async fn enum_encoding() {
        use super::{wl_enum::*, wl_other::*};

        let local = Local::_1Y;
        let remote = Remote::B;

        let received_message = roundtrip_message(EnumMessage { local, remote }).await;

        assert_eq!(local, received_message.local);
        assert_eq!(remote, received_message.remote);
    }

    #[test]
    fn enum_fallback() {
        use cosplay_codec::Enumeration;

        use super::wl_enum::*;

        assert_eq!(Local::_1Y, Local::from_repr(0));
        assert_eq!(0, Local::_1Y.to_repr());

        assert_eq!(Local::Other(123), Local::from_repr(123));
        assert_eq!(123, Local::Other(123).to_repr());
    }

    #[tokio::test]
    async fn bitfield_encoding() {
        use super::wl_bitfields::*;

        let direction = Direction::UP | Direction::DOWN;

        let received_message = roundtrip_message(SomeRequest { direction }).await;

        assert_eq!(direction, received_message.direction);
    }

    #[test]
    fn bitfield_arithmetic() {
        use super::wl_bitfields::*;

        let value = Direction::UP | Direction::DOWN;
        assert_eq!(Direction::UP, value - Direction::DOWN);
        assert_eq!(Direction::DOWN, value & (Direction::DOWN | Direction::LEFT));
    }

    async fn roundtrip_message<M: Message + EncodeMessage + DecodeMessage>(message: M) -> M {
        let object_id = ObjectId::new(OpaqueObjectId(1));

        let mut sink = cosplay_codec::WaylandMessageSink::new(WaylandMemoryBuffer::default());
        sink.send_concrete(object_id, message).await.unwrap();

        let mut stream = cosplay_codec::WaylandMessageStream::new(sink.into_inner());
        let (received_id, received_message) = stream.receive_concrete::<M>().await.unwrap().unwrap();

        assert_eq!(received_id, object_id);

        received_message
    }
}
