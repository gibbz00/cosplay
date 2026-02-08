//! # `async-wayland-core-protocol`

// TODO: autogenerate
mod wl_display {
    pub mod interface {
        pub struct WlDisplay;

        impl async_wayland_common::Interface for WlDisplay {
            type Owner = async_wayland_common::entity::Client;
        }
    }

    pub mod request {
        mod get_registry {
            pub struct GetRegistry {
                pub registry: async_wayland_common::id::NewObjectId<
                    async_wayland_common::entity::Client,
                    crate::wl_registry::WlRegistry,
                >,
            }

            impl async_wayland_common::message::Message for GetRegistry {
                type Interface = crate::wl_display::interface::WlDisplay;
                type Type = async_wayland_common::message::Request;

                const OP_CODE: u16 = 2;

                fn size(&self) -> usize {
                    todo!("use registry.size()")
                }

                fn encode(&self, dst: &mut bytes::BytesMut) {
                    todo!()
                }

                fn decoder() -> impl async_wayland_common::message::MessageDecoder<Message = Self> {
                    GetRegistryDecoder::default()
                }
            }

            #[derive(Default)]
            pub struct GetRegistryDecoder {}

            impl async_wayland_common::message::MessageDecoder for GetRegistryDecoder {
                type Message = GetRegistry;

                fn decode(
                    &mut self,
                    src: &mut bytes::BytesMut,
                ) -> Result<Option<Self::Message>, async_wayland_common::message::MessageDecoderError> {
                    todo!()
                }
            }
        }
        pub use get_registry::GetRegistry;
    }

    pub mod event {
        mod error {
            pub struct Error {
                // TODO: anyid
                message: String,
                object_id: u32,
                code: u32,
            }

            #[derive(Default)]
            pub struct ErrorDecoder {
                stage: ErrorDecoderStage,
                decoders: ErrorArgumentDecoders,
            }

            #[derive(Default)]
            enum ErrorDecoderStage {
                #[default]
                Stage0,
                Stage1(Stage1Args),
                Stage2(Stage2Args),
            }

            struct Stage1Args {
                arg_0: String,
            }

            struct Stage2Args {
                arg_0: String,
                arg_1: u32,
            }

            #[derive(Default)]
            struct ErrorArgumentDecoders {
                string_decoder: async_wayland_common::argument_codec::ArgumentDecoder<String>,
                u32_decoder: async_wayland_common::argument_codec::ArgumentDecoder<u32>,
            }

            impl async_wayland_common::message::MessageDecoder for ErrorDecoder {
                type Message = Error;

                fn decode(
                    &mut self,
                    src: &mut bytes::BytesMut,
                ) -> Result<Option<Error>, async_wayland_common::message::MessageDecoderError> {
                    use async_wayland_common::argument_codec::ArgumentDecode;

                    // we do this to have owned argument values and thus avoid clones
                    let current_stage = std::mem::take(&mut self.stage);

                    match current_stage {
                        ErrorDecoderStage::Stage0 => match self.decoders.string_decoder.decode(src)? {
                            Some(arg_0) => {
                                // First has no Stage0Args to destructure.

                                self.stage = ErrorDecoderStage::Stage1(Stage1Args { arg_0 });

                                self.decode(src)
                            }
                            None => Ok(None),
                        },
                        ErrorDecoderStage::Stage1(stage1_args) => match self.decoders.u32_decoder.decode(src)? {
                            Some(arg_1) => {
                                let Stage1Args { arg_0 } = stage1_args;

                                self.stage = ErrorDecoderStage::Stage2(Stage2Args { arg_0, arg_1 });

                                self.decode(src)
                            }
                            None => {
                                // need to place back current stage into self.stage since std::mem::take was used
                                self.stage = ErrorDecoderStage::Stage1(stage1_args);
                                Ok(None)
                            }
                        },
                        ErrorDecoderStage::Stage2(stage2_args) => match self.decoders.u32_decoder.decode(src)? {
                            Some(arg_2) => {
                                let Stage2Args { arg_0, arg_1 } = stage2_args;

                                // last is a bit special
                                Ok(Some(Error { message: arg_0, object_id: arg_1, code: arg_2 }))
                            }
                            None => {
                                self.stage = ErrorDecoderStage::Stage2(stage2_args);
                                Ok(None)
                            }
                        },
                    }
                }
            }
        }
        pub use error::Error;
    }
}

pub mod wl_registry {
    mod interface {
        pub struct WlRegistry;
    }
    pub use interface::WlRegistry;

    pub mod request {}

    pub mod event {}
}
