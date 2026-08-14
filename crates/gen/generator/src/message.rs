use cosplay_xml::{Argument, ArgumentVariant, Cname, EnumPath, Message};
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

    fn quote(op_code: u16, message: cosplay_xml::Message, ctx: MessageContext) -> proc_macro2::TokenStream {
        let Message { name, description, arguments, .. } = message;

        let doc = DocumentationItem::quote_outer(description.as_ref());

        let ident = IdentifierItem::type_name(&name);

        let interface_ident = IdentifierItem::type_name(ctx.interface_name);

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
                    #[derive(Debug)]
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

            impl ::cosplay_codec::Message for #ident {
                type Interface = #interface_ident;
                const OP_CODE: u16 = #op_code;
            }

            #encode_impl

            #decode_impl
        }
    }

    fn encode_impl(ident: &proc_macro2::Ident, arguments: &[ArgumentItem]) -> proc_macro2::TokenStream {
        let fields = arguments.iter().map(|item| {
            let field_name = &item.field_name;
            quote! { ::cosplay_codec::MarshalArgument::marshal(self.#field_name, _bag); }
        });

        quote! {
            impl ::cosplay_codec::EncodeMessage for #ident {
                fn encode(self, _bag: &mut ::cosplay_codec::ArgumentBag<'_>) {
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
                    quote! { #field_name: ::cosplay_codec::ParseArgument::parse(_bag)?, }
                });

                quote! {
                    Self {
                        #(#fields)*
                    }
                }
            }
        };

        quote! {
            impl ::cosplay_codec::DecodeMessage for #ident {
                fn decode(_bag: &mut ::cosplay_codec::ArgumentBag<'_>) -> Result<Self, ::cosplay_codec::DecodeMessageError> {
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
            doc_comment: DocumentationItem::quote_outer(Some(&description)),
        }
    }

    fn variant_type(variant: ArgumentVariant, ctx: MessageContext) -> proc_macro2::TokenStream {
        return match variant {
            ArgumentVariant::Array => quote! { ::std::vec::Vec<u8> },
            ArgumentVariant::Fd => quote! { ::std::os::fd::OwnedFd },
            ArgumentVariant::I32 { enumeration } => match enumeration {
                Some(path) => Self::enum_path(ctx.interface_name, path, ctx.name_mappings, true),
                None => quote! { i32 },
            },
            ArgumentVariant::U32 { enumeration } => match enumeration {
                Some(path) => Self::enum_path(ctx.interface_name, path, ctx.name_mappings, false),
                None => quote! { u32 },
            },
            ArgumentVariant::Fixed => quote! { ::cosplay_codec::Fixed },
            ArgumentVariant::String { nullable } => maybe_optional(quote! { ::std::string::String }, nullable),
            ArgumentVariant::ObjectId { concrete, nullable } => match concrete {
                None => maybe_optional(quote! { ::cosplay_codec::OpaqueObjectId }, nullable),
                Some(interface_name) => {
                    let path = IdentifierItem::qualified_interface(&interface_name);
                    maybe_optional(quote! { ::cosplay_codec::ObjectId<#path> }, nullable)
                }
            },
            ArgumentVariant::NewObjectId { concrete } => match concrete {
                None => quote! { ::cosplay_codec::OpaqueNewObjectId },
                Some(interface_name) => {
                    let path = IdentifierItem::qualified_interface(&interface_name);
                    quote! { ::cosplay_codec::NewObjectId<#path> }
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

    fn enum_path(parent_interface: &Cname, path: EnumPath, name_mappings: &NameMappings, signed: bool) -> proc_macro2::TokenStream {
        let EnumPath { interface, enumeration } = path;

        let interface_name = interface.as_ref().unwrap_or(parent_interface);

        let mapped_name = name_mappings
            .get(interface_name, &enumeration, ItemType::Enum)
            .unwrap_or(&enumeration);

        let name = IdentifierItem::sanitized_type_name(mapped_name);

        let ident = match interface {
            Some(interface_name) => {
                let module = IdentifierItem::module_name(&interface_name);
                quote! { crate::#module::#name }
            }
            None => quote! { #name },
        };

        let repr = match signed {
            true => quote! { i32 },
            false => quote! { u32 },
        };

        quote! { ::cosplay_codec::EnumArg<#ident, #repr> }
    }
}
