//! Restore inline documentation from external markdown files
//!
//! This module implements the inverse of the migration process, converting
//! external markdown documentation back into inline Rust doc comments.

mod inject;

use crate::discover::ParsedFile;
use proc_macro2::TokenStream;

/// Restores inline documentation by reading markdown files and converting omnidoc attributes
pub fn restore_file(parsed: &ParsedFile, docs_root: &str) -> Option<String> {
    let transformed = inject::inject_all_doc_comments(&parsed.content, docs_root, parsed);

    crate::rewrite::reformat::rewrite_preserving_format_restore(
        &parsed.original_source,
        &transformed.to_string(),
    )
    .ok()
}

fn read_item_markdown(context: &[String], item_name: &str, docs_root: &str) -> Option<String> {
    let mut path_parts = vec![docs_root.to_string()];
    path_parts.extend(context.iter().cloned());
    path_parts.push(format!("{}.md", item_name));

    let md_path = path_parts.join("/");
    std::fs::read_to_string(&md_path).ok()
}

fn read_module_doc(parsed: &ParsedFile, docs_root: &str) -> Option<String> {
    let module_path = syncdoc_core::path_utils::extract_module_path(&parsed.path.to_string_lossy());

    let file_stem = parsed
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module");

    let md_path = if module_path.is_empty() {
        format!("{}/{}.md", docs_root, file_stem)
    } else {
        format!("{}/{}.md", docs_root, module_path)
    };

    std::fs::read_to_string(&md_path).ok()
}

fn generate_doc_comments(content: &str) -> TokenStream {
    use quote::quote;
    let lines: Vec<_> = content.trim_end().lines().collect();
    let mut output = TokenStream::new();

    for line in lines {
        let comment = format!("/// {}", line);
        output.extend(quote! { #[doc = #comment] });
    }

    output
}

fn generate_module_doc_comments(content: &str) -> TokenStream {
    use quote::quote;
    let lines: Vec<_> = content.trim_end().lines().collect();
    let mut output = TokenStream::new();

    for line in lines {
        let comment = format!("//! {}", line);
        output.extend(quote! { #[doc = #comment] });
    }

    output
}

fn is_omnidoc_attr(attr: &syncdoc_core::parse::Attribute) -> bool {
    use unsynn::ToTokens;
    let mut ts = TokenStream::new();
    attr.to_tokens(&mut ts);
    let s = ts.to_string().replace(' ', "");
    s.contains("omnidoc") || s.contains("syncdoc::omnidoc")
}

fn is_module_doc_macro(inner_attrs: &unsynn::Many<syncdoc_core::parse::InnerAttribute>) -> bool {
    use unsynn::ToTokens;
    for attr in &inner_attrs.0 {
        let mut ts = TokenStream::new();
        attr.value.to_tokens(&mut ts);
        let s = ts.to_string().replace(' ', "");
        if s.contains("module_doc!") {
            return true;
        }
    }
    false
}

#[cfg(test)]
#[path = "tests/restore.rs"]
mod tests;
