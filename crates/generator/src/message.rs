use async_wayland_xml::{Argument, ArgumentVariant, Cname, Message};
use quote::quote;

use crate::*;

pub struct MessageItem;

#[derive(Clone, Copy)]
pub struct MessageContext<'a> {
    pub interface_name: &'a Cname,
    pub name_mappings: &'a NameMappings,
}

impl MessageItem {
    pub fn quote_list(messages: Vec<Message>, ctx: MessageContext) -> Vec<proc_macro2::TokenStream> {
        messages
            .into_iter()
            .enumerate()
            .map(|(op_code, message)| Self::quote(op_code as u16, message, ctx))
            .collect()
    }

    fn quote(op_code: u16, message: async_wayland_xml::Message, ctx: MessageContext) -> proc_macro2::TokenStream {
        let Message { name, destructor, since, deprecated_since, description, arguments } = message;

        let doc = Documentation::quote_outer(description.as_ref());

        let ident = IdentifierItem::type_name(&name);

        let argument_items = arguments.into_iter().map(|var| ArgumentItem::new(var, ctx)).collect::<Vec<_>>();

        let struct_declaration = match argument_items.is_empty() {
            true => quote! {
                #doc
                pub struct #ident;
            },
            false => {
                let fields = argument_items.iter().map(|ArgumentItem { field_name, rust_type, doc_comment }| {
                    quote! {
                        #doc_comment
                        pub #field_name: #rust_type,
                    }
                });

                quote! {
                    #doc
                    pub struct #ident {
                        #(#fields)*
                    }
                }
            }
        };

        let encode_impl = Self::encode_impl(&ident, &argument_items);

        let decode_impl = Self::decode_impl(&ident, &argument_items);

        quote! {
            #struct_declaration

            impl ::async_wayland_codec::Message for #ident {
                const OP_CODE: u16 = #op_code;
            }

            #encode_impl

            #decode_impl
        }
    }

    fn encode_impl(ident: &proc_macro2::Ident, arguments: &[ArgumentItem]) -> proc_macro2::TokenStream {
        let fields = arguments.iter().map(|item| {
            let field_name = &item.field_name;
            quote! { self.#field_name.marshal(bag); }
        });

        quote! {
            impl ::async_wayland_codec::EncodeMessage for #ident {
                fn encode(self, bag: &mut ::async_wayland_codec::ArgumentBag<'_>) {
                    use ::async_wayland_codec::MarshalArgument as _;
                    #(#fields)*
                }
            }
        }
    }

    fn decode_impl(ident: &proc_macro2::Ident, arguments: &[ArgumentItem]) -> proc_macro2::TokenStream {
        let body = match arguments.is_empty() {
            true => quote! { Self },
            false => {
                let fields = arguments.iter().map(|item| {
                    let field_name = &item.field_name;
                    quote! { #field_name: ::async_wayland_codec::ParseArgument::parse(bag)?, }
                });

                quote! {
                    Self {
                        #(#fields)*
                    }
                }
            }
        };

        quote! {
            impl ::async_wayland_codec::DecodeMessage for #ident {
                fn decode(bag: &mut ::async_wayland_codec::ArgumentBag<'_>) -> Result<Self, ::async_wayland_codec::DecodeMessageError> {
                    Ok(#body)
                }
            }
        }
    }
}

struct ArgumentItem {
    field_name: proc_macro2::Ident,
    rust_type: proc_macro2::TokenStream,
    doc_comment: Option<proc_macro2::TokenStream>,
}

impl ArgumentItem {
    fn new(argument: Argument, ctx: MessageContext) -> Self {
        let Argument { name, variant, description } = argument;

        Self {
            field_name: IdentifierItem::field_name(&name),
            rust_type: Self::variant_type(variant, ctx),
            doc_comment: Documentation::quote_outer(Some(&description)),
        }
    }

    fn variant_type(variant: ArgumentVariant, ctx: MessageContext) -> proc_macro2::TokenStream {
        return match variant {
            ArgumentVariant::Array => quote! { ::std::vec::Vec<u8> },
            ArgumentVariant::Fd => quote! { ::std::os::fd::OwnedFd },
            ArgumentVariant::I32 { enumeration } => match enumeration {
                Some(path) => IdentifierItem::enum_path(ctx.interface_name, path, ctx.name_mappings),
                None => quote! { i32 },
            },
            ArgumentVariant::U32 { enumeration } => match enumeration {
                Some(path) => IdentifierItem::enum_path(ctx.interface_name, path, ctx.name_mappings),
                None => quote! { u32 },
            },
            ArgumentVariant::Fixed => quote! { ::async_wayland_codec::Fixed },
            ArgumentVariant::String { nullable } => maybe_optional(quote! { ::std::string::String }, nullable),
            ArgumentVariant::ObjectId { concrete, nullable } => match concrete {
                None => maybe_optional(quote! { ::async_wayland_codec::OpaqueObjectId }, nullable),
                Some(interface_name) => {
                    let path = IdentifierItem::qualified_interface(&interface_name);
                    maybe_optional(quote! { ::async_wayland_codec::ObjectId<#path> }, nullable)
                }
            },
            ArgumentVariant::NewObjectId { concrete } => match concrete {
                None => quote! { ::async_wayland_codec::OpaqueNewObjectId },
                Some(interface_name) => {
                    let path = IdentifierItem::qualified_interface(&interface_name);
                    quote! { ::async_wayland_codec::NewObjectId<#path> }
                }
            },
        };

        fn maybe_optional(path: proc_macro2::TokenStream, nullable: bool) -> proc_macro2::TokenStream {
            match nullable {
                true => quote! { ::std::option::Option<#path> },
                false => path,
            }
        }
    }
}
