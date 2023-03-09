use proc_macro::TokenStream;
use syn;

#[proc_macro_derive(Builder)]  // Use it with #[derive(Builder)] in main crate
pub fn derive(input: TokenStream) -> TokenStream {
    let st = syn::parse_macro_input!(input as syn::DeriveInput);
    TokenStream::new()
}
