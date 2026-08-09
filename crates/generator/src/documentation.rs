use async_wayland_xml::Description;
use quote::quote;

pub struct Documentation;

impl Documentation {
    pub fn quote_outer(description: Option<Description>) -> Option<proc_macro2::TokenStream> {
        description.map(|description| Self::quote(description, false))
    }

    pub fn quote(description: Description, inner_attribute: bool) -> proc_macro2::TokenStream {
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
            true => quote! { #( #[doc = #lines] )* },
            false => quote! { #( #![doc = #lines] )* },
        }
    }
}
