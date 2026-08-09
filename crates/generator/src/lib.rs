//! # `async-wayland-generator` - Generate `async-wayland-codec` trait implementations from XML protocol definitions.

use anyhow::Context;
use async_wayland_xml::{Interface, Protocol};
use quote::quote;

mod documentation;
pub(crate) use documentation::Documentation;

/// Convert a wayland protocol into Rust source code implementing `async-wayland-codec` traits.
pub fn run(protocol: Protocol) -> anyhow::Result<String> {
    let Protocol { description, interfaces, .. } = protocol;

    let description_comment = Documentation::quote_outer(description);

    let interface_modules = interfaces.into_iter().map(prepare_interface);

    // TODO: iterate over interfaces

    let src = quote! {
        #description_comment

        #(#interface_modules)*
    };

    format_src(src)
}

fn format_src(src: proc_macro2::TokenStream) -> anyhow::Result<String> {
    let file = syn::parse2::<syn::File>(src).context("Failed to parse token stream.")?;
    let formatted = prettyplease::unparse(&file);
    Ok(formatted)
}

fn prepare_interface(interface: Interface) -> proc_macro2::TokenStream {
    let Interface { name, version, frozen, description, requests, events, enums } = interface;

    let name = quote::format_ident!("{}", name.as_ref());
    let description_comment = Documentation::quote_outer(description);

    quote! {
        pub mod #name {
            #description_comment
        }
    }
}
