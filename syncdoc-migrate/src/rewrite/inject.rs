// syncdoc-migrate/src/rewrite/inject.rs

use proc_macro2::TokenStream;
use quote::quote;

/// Injects `#[omnidoc(path = "...")]` attribute into an item's token stream
pub fn inject_omnidoc_attr(item: TokenStream, docs_root: &str) -> TokenStream {
    let attr = quote! {
        #[syncdoc::omnidoc(path = #docs_root)]
    };

    // Simply prepend the attribute before the entire item
    let mut output = TokenStream::new();
    output.extend(attr);
    output.extend(item);
    output
}
