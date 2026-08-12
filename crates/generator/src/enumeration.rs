use async_wayland_xml::{
    Cname, Enum, EnumEntry,
    utils::{EnumRepr, EnumReprMap},
};
use quote::quote;

use crate::*;

pub struct EnumItem;

#[derive(Clone, Copy)]
pub struct EnumContext<'a> {
    pub interface_name: &'a Cname,
    pub repr_map: &'a EnumReprMap,
    pub name_mappings: &'a NameMappings,
}

impl EnumItem {
    pub fn quote_list(enums: Vec<Enum>, ctx: EnumContext) -> Vec<proc_macro2::TokenStream> {
        enums.into_iter().map(|enumeration| Self::quote(enumeration, ctx)).collect()
    }

    fn quote(enumeration: Enum, ctx: EnumContext) -> proc_macro2::TokenStream {
        // FIXME: switch on bitfield
        let Enum { name, bitfield, since, description, entries } = enumeration;

        let doc = Documentation::quote_outer(description.as_ref());

        let Some(repr) = ctx.repr_map.get(ctx.interface_name, &name) else {
            // Enum part of another interface or protocol.
            return Default::default();
        };

        let translated_name = ctx.name_mappings.get(ctx.interface_name, &name, ItemType::Enum).unwrap_or(&name);
        let enum_ident = IdentifierItem::sanitized_type_name(translated_name);

        let enum_fields = entries.iter().map(|entry| {
            let field_ident = IdentifierItem::sanitized_type_name(&entry.name);

            let doc = Documentation::quote_outer(Some(&entry.description));

            quote! {
                #doc
                #field_ident,
            }
        });

        let pairs = Self::variant_value_pairs(&enum_ident, &entries);

        let from_repr_fields = pairs.iter().map(|(variant, value)| {
            quote! { #value => #variant, }
        });

        let to_repr_fields = pairs.iter().map(|(variant, value)| {
            quote! { #variant => #value, }
        });

        let repr_ident = match repr {
            EnumRepr::U32 => quote! { u32 },
            EnumRepr::I32 => quote! { i32 },
        };

        quote! {
            #doc
            pub enum #enum_ident {
                #(#enum_fields)*
                /// Fallback variant undocumented entry values.
                Other(#repr_ident)
            }

            impl ::async_wayland_codec::Enumeration for #enum_ident {
                type Repr = #repr_ident;

                fn from_repr(repr: Self::Repr) -> Self {
                    match repr {
                        #(#from_repr_fields)*
                        other => #enum_ident::Other(other),
                    }
                }

                fn to_repr(&self) -> Self::Repr {
                    match self {
                        #(#to_repr_fields)*
                        #enum_ident::Other(other) => *other,
                    }
                }
            }
        }
    }

    fn variant_value_pairs(
        enum_ident: &proc_macro2::Ident,
        entries: &[EnumEntry],
    ) -> Vec<(proc_macro2::TokenStream, proc_macro2::Literal)> {
        entries
            .iter()
            .map(|entry| {
                let field_ident = IdentifierItem::sanitized_type_name(&entry.name);
                let value = proc_macro2::Literal::i64_unsuffixed(entry.value);

                let variant = quote! {
                    #enum_ident::#field_ident
                };

                (variant, value)
            })
            .collect()
    }
}
