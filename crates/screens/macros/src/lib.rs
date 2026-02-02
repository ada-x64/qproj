extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Proc macro deriving the Screen trait.
#[proc_macro_derive(Screen)]
pub fn derive_screen(ts: TokenStream) -> TokenStream {
    let input = parse_macro_input!(ts as DeriveInput);
    let ident = input.ident;
    quote! {
        impl Screen for #ident {}
    }
    .into()
}
