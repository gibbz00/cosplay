use async_wayland_xml::{Argument, ArgumentVariant, Message};
use quote::quote;

use crate::*;

pub struct MessageItem;

struct ArgumentItem {
    field_name: proc_macro2::Ident,
    rust_type: proc_macro2::TokenStream,
    doc_comment: Option<proc_macro2::TokenStream>,
}

impl ArgumentItem {
    fn new(argument: Argument) -> Self {
        let Argument { name, variant, description } = argument;

        Self {
            field_name: IdentifierItem::field_name(name),
            rust_type: Self::variant_type(variant),
            doc_comment: Documentation::quote_outer(Some(description)),
        }
    }

    fn variant_type(variant: ArgumentVariant) -> proc_macro2::TokenStream {
        return match variant {
            ArgumentVariant::Array => quote! { ::std::vec::Vec<u8> },
            ArgumentVariant::Fd => quote! { ::std::os::fd::OwnedFd },
            ArgumentVariant::I32 { enumeration } => match enumeration {
                Some(path) => IdentifierItem::enum_path(path),
                None => quote! { i32 },
            },
            ArgumentVariant::U32 { enumeration } => match enumeration {
                None => quote! { u32 },
                Some(path) => IdentifierItem::enum_path(path),
            },
            ArgumentVariant::Fixed => quote! { ::async_wayland_codec::Fixed },
            ArgumentVariant::String { nullable } => maybe_optional(quote! { ::std::string::String }, nullable),
            ArgumentVariant::ObjectId { concrete, nullable } => match concrete {
                None => maybe_optional(quote! { ::async_wayland_codec::OpaqueObjectId }, nullable),
                Some(interface_name) => {
                    let path = IdentifierItem::qualified_interface(interface_name);
                    maybe_optional(quote! { ::async_wayland_codec::ObjectId<#path> }, nullable)
                }
            },
            ArgumentVariant::NewObjectId { concrete } => match concrete {
                None => quote! { ::async_wayland_codec::OpaqueNewObjectId },
                Some(interface_name) => {
                    let path = IdentifierItem::qualified_interface(interface_name);
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

impl MessageItem {
    pub fn quote_list(messages: Vec<Message>) -> impl Iterator<Item = proc_macro2::TokenStream> {
        messages
            .into_iter()
            .enumerate()
            .map(|(op_code, message)| MessageItem::quote(op_code, message))
    }

    pub fn quote(op_code: usize, message: async_wayland_xml::Message) -> proc_macro2::TokenStream {
        let Message { name, destructor, since, deprecated_since, description, arguments } = message;

        let doc = Documentation::quote_outer(description);

        let ident = IdentifierItem::type_name(name);

        let argument_items = arguments.into_iter().map(ArgumentItem::new).collect::<Vec<_>>();

        let item = match argument_items.is_empty() {
            true => quote! {
                #doc
                pub struct #ident;
            },
            false => {
                let fields = argument_items.iter().map(|ArgumentItem { field_name, rust_type, doc_comment }| {
                    quote! {
                        #doc_comment
                        #field_name: #rust_type,
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

        // TODO: implement message, encode decode

        quote! {
            #item
        }
    }
}
