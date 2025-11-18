// syncdoc-migrate/src/rewrite/inject.rs

use crate::config::DocsPathMode;
use proc_macro2::TokenStream;
use quote::quote;

/// Checks if a token stream already contains an omnidoc attribute
fn has_omnidoc_attr(item: &TokenStream) -> bool {
    let item_str = item.to_string().replace(" ", "");
    item_str.contains("#[omnidoc") || item_str.contains("#[syncdoc::omnidoc")
}

/// Checks if a token stream already contains a module_doc macro
pub fn has_module_doc_macro(item: &TokenStream) -> bool {
    let item_str = item.to_string().replace(" ", "");
    item_str.contains("module_doc!") && item_str.contains("#![doc")
}

/// Injects `#[omnidoc(path = "...")]` attribute into an item's token stream
///
/// If `mode` is `TomlConfig`, omits the path parameter.
/// If `mode` is `InlinePaths`, includes the path parameter.
///
/// **Idempotent**: Returns the original item unchanged if it already has an omnidoc attribute.
pub fn inject_omnidoc_attr(item: TokenStream, docs_root: &str, mode: DocsPathMode) -> TokenStream {
    // Skip if already has omnidoc attribute (idempotency)
    if has_omnidoc_attr(&item) {
        return item;
    }

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
///
/// **Idempotent**: Returns empty TokenStream if module_doc macro already exists.
pub fn inject_module_doc_attr(docs_root: &str, mode: DocsPathMode) -> TokenStream {
    // Note: We can't check the existing content here since this creates a new attribute.
    // The idempotency check needs to happen at the call site in rewrite.rs
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
#[path = "../tests/inject.rs"]
mod inject_tests;
