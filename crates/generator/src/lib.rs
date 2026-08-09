//! # `async-wayland-generator` - Generate `async-wayland-codec` trait implementations from XML protocol definitions.

use anyhow::Context;
use async_wayland_xml::{Description, Interface, Protocol};
use quote::quote;

/// Convert a wayland protocol into Rust source code implementing `async-wayland-codec` traits.
pub fn run(protocol: Protocol) -> anyhow::Result<String> {
    let Protocol { description, interfaces, .. } = protocol;

    let description_comment = outer_description(description);

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
    let description_comment = outer_description(description);

    quote! {
        pub mod #name {
            #description_comment
        }
    }
}

fn outer_description(description: Option<Description>) -> Option<proc_macro2::TokenStream> {
    description.map(|description| prepare_description(description, false))
}

fn prepare_description(description: Description, inner_attribute: bool) -> proc_macro2::TokenStream {
    let Description { summary, text } = description;

    let mut lines = Vec::<String>::new();

    if let Some(summary) = &summary {
        let mut chars = summary.chars();

        if let Some(first) = chars.next() {
            let title = format!(" {}{}.", first.to_uppercase(), chars.as_str());
            lines.push(title);
        };
    }

    if let Some(text) = &text {
        for line in text.lines() {
            lines.push(format!(" {}", line.trim()));
        }
    }

    match inner_attribute {
        true => quote! {
            #( #[doc = #lines] )*
        },
        false => quote! {
            #( #![doc = #lines] )*
        },
    }
}
