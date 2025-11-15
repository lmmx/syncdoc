// syncdoc-migrate/src/rewrite/inject.rs

use crate::config::DocsPathMode;
use proc_macro2::TokenStream;
use quote::quote;

/// Injects `#[omnidoc(path = "...")]` attribute into an item's token stream
///
/// If `mode` is `TomlConfig`, omits the path parameter.
/// If `mode` is `InlinePaths`, includes the path parameter.
pub fn inject_omnidoc_attr(item: TokenStream, docs_root: &str, mode: DocsPathMode) -> TokenStream {
    let attr = match mode {
        DocsPathMode::InlinePaths => quote! {
            #[syncdoc::omnidoc(path = #docs_root)]
        },
        DocsPathMode::TomlConfig => quote! {
            #[syncdoc::omnidoc]
        },
    };

    // Simply prepend the attribute before the entire item
    let mut output = TokenStream::new();
    output.extend(attr);
    output.extend(item);
    output
}

/// Injects `#![doc = syncdoc::module_doc!()]` for module-level documentation
///
/// If `mode` is `TomlConfig`, omits the path parameter.
/// If `mode` is `InlinePaths`, includes the path parameter.
pub fn inject_module_doc_attr(docs_root: &str, mode: DocsPathMode) -> TokenStream {
    match mode {
        DocsPathMode::InlinePaths => quote! {
            #![doc = syncdoc::module_doc!(path = #docs_root)]
        },
        DocsPathMode::TomlConfig => quote! {
            #![doc = syncdoc::module_doc!()]
        },
    }
}

#[cfg(test)]
mod inject_tests;
