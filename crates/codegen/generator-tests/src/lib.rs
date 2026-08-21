#![allow(missing_docs)]

include!(concat!(env!("OUT_DIR"), "/combined.rs"));

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use cosplay_codec::*;

    #[test]
    fn rename() {
        use super::rename::*;
        // NB: Pixelformat would normally be called Format without the rename,
        // which would then cause a compiler error.
        let _ = Format { format: PixelFormat::A.into() };
    }

    #[test]
    fn interface_name() {
        use super::wl_encoding::*;

        assert_eq!("wl_encoding", <WlEncoding as cosplay_codec::Interface>::NAME);
    }

    #[test]
    fn interface_version() {
        use super::wl_encoding::*;

        assert_eq!(123, <WlEncoding as cosplay_codec::Interface>::VERSION);
    }

    #[test]
    fn message_name() {
        use super::opcodes::*;

        assert_eq!("ev_c", <EvC as cosplay_codec::Message>::NAME);
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

        let id = ObjectId::<WlB>::new(OpaqueObjectId::new(2));

        let received_message = roundtrip_message(Request { id }).await;

        assert_eq!(id, received_message.id);
    }

    #[tokio::test]
    async fn enum_encoding() {
        use super::{wl_enum::*, wl_other::*};

        let local = Local::_1Y.into();
        let remote = Remote::B.into();

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

        let direction = (Direction::UP | Direction::DOWN).into();

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

    #[tokio::test]
    async fn inbound_requests() {
        use super::opcodes::*;

        let request = roundtrip_inbound::<Request, _>(ReqA).await.unwrap();
        assert_matches!(request, OpcodesRequest::ReqA(_));

        let request = roundtrip_inbound::<Request, _>(ReqB).await.unwrap();
        assert_matches!(request, OpcodesRequest::ReqB(_));
    }

    #[tokio::test]
    async fn inbound_events() {
        use super::opcodes::*;

        let event = roundtrip_inbound::<Event, _>(EvC).await.unwrap();
        assert_matches!(event, OpcodesEvent::EvC(_));
    }

    #[tokio::test]
    async fn inbound_unknown_error() {
        use super::opcodes::*;

        let error = roundtrip_inbound::<Event, _>(ReqB).await.unwrap_err();
        assert_matches!(error, IntoInboundError::UnknownOpcode(_));
    }

    async fn roundtrip_message<M: Message + EncodeMessage + DecodeMessage>(message: M) -> M {
        let object_id = ObjectId::new(OpaqueObjectId::new(1));

        let mut sink = WaylandMessageSink::new(WaylandMemoryBuffer::default());

        sink.send_concrete(object_id, message).await.unwrap();

        let mut stream = WaylandMessageStream::new(sink.into_inner());

        let received_message = stream.receive_opaque().await.unwrap().unwrap();
        received_message.matches::<M>(object_id).unwrap();

        received_message.into_concrete::<M>().unwrap()
    }

    async fn roundtrip_inbound<D: Direction, M: Message + EncodeMessage>(
        message: M,
    ) -> Result<<M::Interface as Inbound<D>>::Enum, IntoInboundError>
    where
        M::Interface: Inbound<D>,
    {
        let object_id = ObjectId::new(OpaqueObjectId::new(1));

        let mut sink = WaylandMessageSink::new(WaylandMemoryBuffer::default());

        sink.send_concrete(object_id, message).await.unwrap();

        let mut stream = WaylandMessageStream::new(sink.into_inner());

        let message = stream.receive_opaque().await.unwrap().unwrap();

        <M::Interface>::from_opaque(message)
    }
}

// NB: Used to test if the output of `external.xml` compiles. Do not remove.
mod external_mod {
    pub mod wl_ext {
        pub struct WlExt;

        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum ExtEnum {
            Other(u32),
        }

        impl cosplay_codec::Enumeration<u32> for ExtEnum {
            fn from_repr(repr: u32) -> Self {
                Self::Other(repr)
            }

            fn to_repr(&self) -> u32 {
                match self {
                    ExtEnum::Other(repr) => *repr,
                }
            }
        }
    }
}
