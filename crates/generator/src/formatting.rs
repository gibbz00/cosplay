pub struct Formatter;

impl Formatter {
    pub fn format(src: proc_macro2::TokenStream) -> syn::Result<String> {
        syn::parse2::<syn::File>(src).map(|file| prettyplease::unparse(&file))
    }
}
