use proc_macro::TokenStream;

mod derive_diff;
mod derive_fields;

#[proc_macro_derive(Fields, attributes(differs))]
pub fn diff_fields(input: TokenStream) -> TokenStream {
    derive_fields::derive_fields_impl(input)
}

#[proc_macro_derive(Diff, attributes(differs))]
pub fn diff_changes(input: TokenStream) -> TokenStream {
    derive_diff::derive_diff_impl(input)
}
