use async_wayland_xml::{
    Cname, Enum, EnumEntry,
    utils::{EnumRepr, EnumReprMap},
};
use quote::quote;

use crate::{Documentation, IdentifierItem};

pub struct EnumItem;

impl EnumItem {
    pub fn quote_list(interface_name: &Cname, enums: Vec<Enum>, repr_map: &EnumReprMap) -> impl Iterator<Item = proc_macro2::TokenStream> {
        enums
            .into_iter()
            .map(|enumeration| Self::quote(interface_name, enumeration, repr_map))
    }

    fn quote(interface_name: &Cname, enumeration: Enum, repr_map: &EnumReprMap) -> proc_macro2::TokenStream {
        // FIXME: switch on bitfield
        let Enum { name, bitfield, since, description, entries } = enumeration;

        let doc = Documentation::quote_outer(description.as_ref());

        let Some(repr) = repr_map.get(interface_name, &name) else {
            // Enum part of another interface or protocol.
            return Default::default();
        };

        let enum_ident = IdentifierItem::sanitized_type_name(&name);

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
