use cosplay_xml::{Cname, Interface, Message};
use quote::quote;

use crate::*;

pub struct InterfaceItem;

#[derive(Clone, Copy)]
pub struct InterfaceContext<'a> {
    pub external_interfaces: &'a ExternalInterfaces,
    pub name_mappings: &'a NameMappings,
}

impl InterfaceItem {
    pub fn quote(interface: Interface, ctx: InterfaceContext) -> proc_macro2::TokenStream {
        let Interface { name, description, requests, events, enums, version, .. } = interface;

        let module_name = IdentifierItem::module_name(&name);

        let doc = DocumentationItem::quote_outer(description.as_ref());

        let ident = IdentifierItem::type_name(&name);

        let trait_impl = Self::trait_impl(&ident, &name, version.as_ref().get());

        let message_ctx = MessageContext {
            interface_name: &name,
            external_interfaces: ctx.external_interfaces,
            name_mappings: ctx.name_mappings,
        };

        let inbound_requests = Self::inbound_request_impl(&ident, &requests);
        let requests = MessageItem::quote_list(MessageType::Request, requests, message_ctx);

        let inbound_events = Self::inbound_events_impl(&ident, &events);
        let events = MessageItem::quote_list(MessageType::Event, events, message_ctx);

        let enums = EnumItem::quote_list(enums, EnumContext { interface_name: &name, name_mappings: ctx.name_mappings });

        quote! {
            pub mod #module_name {
                #doc
                pub struct #ident;

                #trait_impl

                #(#requests)*

                #inbound_requests

                #(#events)*

                #inbound_events

                #(#enums)*
            }
        }
    }

    fn trait_impl(ident: &proc_macro2::Ident, name: &Cname, version: u32) -> proc_macro2::TokenStream {
        let name = name.as_ref();
        quote! {
            impl ::cosplay_codec::Interface for #ident {
                const NAME: &str = #name;
                const VERSION: u32 = #version;
            }
        }
    }

    fn inbound_request_impl(interface_ident: &proc_macro2::Ident, messages: &[Message]) -> proc_macro2::TokenStream {
        Self::inbound_impl(
            interface_ident,
            &quote::format_ident!("{interface_ident}Request"),
            &quote! { ::cosplay_codec::Request },
            messages,
        )
    }

    fn inbound_events_impl(interface_ident: &proc_macro2::Ident, messages: &[Message]) -> proc_macro2::TokenStream {
        Self::inbound_impl(
            interface_ident,
            &quote::format_ident!("{interface_ident}Event"),
            &quote! { ::cosplay_codec::Event },
            messages,
        )
    }

    fn inbound_impl(
        interface_ident: &proc_macro2::Ident,
        enum_ident: &proc_macro2::Ident,
        direction: &proc_macro2::TokenStream,
        messages: &[Message],
    ) -> proc_macro2::TokenStream {
        let message_idents = messages
            .iter()
            .map(|message| IdentifierItem::type_name(&message.name))
            .collect::<Vec<_>>();

        quote! {
            #[derive(Debug)]
            pub enum #enum_ident {
                #(#message_idents(#message_idents),)*
            }

            impl ::cosplay_codec::Inbound<#direction> for #interface_ident {
                type Enum = #enum_ident;

                fn from_opaque(message: ::cosplay_codec::OpaqueMessage) -> Result<Self::Enum, ::cosplay_codec::IntoInboundError> {
                    match message.opcode() {
                        #(
                            <#message_idents as ::cosplay_codec::Message>::OP_CODE => message
                                .into_concrete()
                                .map(#enum_ident::#message_idents)
                                .map_err(::cosplay_codec::IntoInboundError::from_decode::<#message_idents>),
                        )*
                        _ => Err(::cosplay_codec::IntoInboundError::UnknownOpcode(message)),
                    }
                }
            }
        }
    }
}
