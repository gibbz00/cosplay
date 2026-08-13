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
        let Enum { name, bitfield, description, entries, .. } = enumeration;

        let doc = Documentation::quote_outer(description.as_ref());

        let Some(repr) = ctx.repr_map.get(ctx.interface_name, &name) else {
            // Enum part of another interface or protocol.
            return Default::default();
        };

        let repr_ident = match repr {
            EnumRepr::U32 => quote! { u32 },
            EnumRepr::I32 => quote! { i32 },
        };

        let translated_name = ctx.name_mappings.get(ctx.interface_name, &name, ItemType::Enum).unwrap_or(&name);
        let enum_ident = IdentifierItem::sanitized_type_name(translated_name);

        match bitfield {
            true => Self::quote_bitfield(doc, &enum_ident, repr_ident, entries),
            false => Self::quote_enum(doc, &enum_ident, repr_ident, entries),
        }
    }

    fn quote_enum(
        doc: Option<proc_macro2::TokenStream>,
        enum_ident: &proc_macro2::Ident,
        repr_ident: proc_macro2::TokenStream,
        entries: Vec<EnumEntry>,
    ) -> proc_macro2::TokenStream {
        let enum_fields = entries.iter().map(|entry| {
            let field_ident = IdentifierItem::sanitized_type_name(&entry.name);

            let doc = Documentation::quote_outer(Some(&entry.description));

            quote! {
                #doc
                #field_ident,
            }
        });

        let pairs = Self::variant_value_pairs(enum_ident, &entries);

        let from_repr_fields = pairs.iter().map(|(variant, value)| {
            quote! { #value => #variant, }
        });

        let to_repr_fields = pairs.iter().map(|(variant, value)| {
            quote! { #variant => #value, }
        });

        quote! {
            #doc
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum #enum_ident {
                #(#enum_fields)*
                /// Fallback variant for undocumented entry values.
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

    fn quote_bitfield(
        doc: Option<proc_macro2::TokenStream>,
        enum_ident: &proc_macro2::Ident,
        repr_ident: proc_macro2::TokenStream,
        entries: Vec<EnumEntry>,
    ) -> proc_macro2::TokenStream {
        let entry_consts = entries.iter().map(|entry| {
            let doc = Documentation::quote_outer(Some(&entry.description));

            let const_ident = IdentifierItem::sanitized_const_name(&entry.name);

            let value = proc_macro2::Literal::i64_unsuffixed(entry.value);

            quote! {
                #doc
                pub const #const_ident: #enum_ident = #enum_ident(#value);
            }
        });

        quote! {
            #doc
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct #enum_ident(#repr_ident);

            impl #enum_ident {
                #(#entry_consts)*

                /// Create an empty instance with its internal value set to zero.
                pub fn empty() -> Self {
                    #enum_ident(0)
                }

                /// Get the internal bitfield value.
                pub fn bits(&self) -> u32 {
                    self.0
                }
            }

            impl ::std::ops::BitOr for #enum_ident {
                type Output = Self;

                fn bitor(self, rhs: Self) -> Self {
                    #enum_ident(self.0 | rhs.0)
                }
            }

            impl ::std::ops::BitAnd for #enum_ident {
                type Output = Self;

                fn bitand(self, rhs: Self) -> Self {
                    #enum_ident(self.0 & rhs.0)
                }
            }

            impl ::std::ops::Sub for #enum_ident {
                type Output = Self;

                fn sub(self, rhs: Self) -> Self {
                    #enum_ident(self.0 & !rhs.0)
                }
            }

            impl ::async_wayland_codec::Enumeration for #enum_ident {
                type Repr = #repr_ident;

                fn from_repr(repr: Self::Repr) -> Self {
                    Self(repr)
                }

                fn to_repr(&self) -> Self::Repr {
                    self.0
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
