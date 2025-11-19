// syncdoc-migrate/src/rewrite.rs

pub(crate) mod inject;
pub mod reformat;
pub(crate) mod strip;

pub use inject::{inject_module_doc_attr, inject_omnidoc_attr};
pub use strip::{strip_doc_attrs, strip_inner_doc_attrs};
pub(crate) use unsynn::*;

use crate::config::DocsPathMode;
use crate::discover::ParsedFile;
use crate::rewrite::inject::has_module_doc_macro;
pub(crate) use proc_macro2::TokenStream;
use reformat::rewrite_preserving_format;
use strip::strip_doc_attrs_from_items;
use syncdoc_core::parse::ModuleItem;

pub fn rewrite_file(
    parsed: &ParsedFile,
    docs_root: &str,
    docs_mode: DocsPathMode,
    strip: bool,
    annotate: bool,
) -> Option<String> {
    if !strip && !annotate {
        return None;
    }

    let mut output = if strip {
        strip_doc_attrs_from_items(&parsed.content)
    } else {
        let mut ts = TokenStream::new();
        quote::ToTokens::to_tokens(&parsed.content, &mut ts);
        ts
    };

    if annotate {
        // Re-parse to inject annotations
        if let Ok(content) = output
            .clone()
            .into_token_iter()
            .parse::<syncdoc_core::parse::ModuleContent>()
        {
            let mut annotated = TokenStream::new();

            // Check if module_doc already exists
            let has_module_doc = if let Some(inner_attrs) = &content.inner_attrs {
                let mut temp_ts = TokenStream::new();
                unsynn::ToTokens::to_tokens(inner_attrs, &mut temp_ts);
                has_module_doc_macro(&temp_ts)
            } else {
                false
            };

            // Inject module_doc first, if needed (for inner docs if any existed AND not already present)
            if !has_module_doc
                && (content.inner_attrs.is_some() || parsed.content.inner_attrs.is_some())
            {
                annotated.extend(inject_module_doc_attr(docs_root, docs_mode));
            }

            // Then add any remaining non-doc inner attributes
            let stripped_inner = strip_inner_doc_attrs(&content.inner_attrs);
            for attr in stripped_inner {
                quote::ToTokens::to_tokens(&attr, &mut annotated);
            }

            // Then handle regular items
            for item_delimited in &content.items.0 {
                let mut item_ts = TokenStream::new();
                quote::ToTokens::to_tokens(&item_delimited.value, &mut item_ts);

                let should_annotate = matches!(
                    &item_delimited.value,
                    ModuleItem::Function(_)
                        | ModuleItem::Enum(_)
                        | ModuleItem::Struct(_)
                        | ModuleItem::Module(_)
                        | ModuleItem::Trait(_)
                        | ModuleItem::ImplBlock(_)
                        | ModuleItem::TypeAlias(_)
                        | ModuleItem::Const(_)
                        | ModuleItem::Static(_)
                );

                if should_annotate {
                    // inject_omnidoc_attr now handles idempotency internally
                    annotated.extend(inject_omnidoc_attr(item_ts, docs_root, docs_mode));
                } else {
                    annotated.extend(item_ts);
                }
            }
            output = annotated;
        }
    }

    let transformed = output.to_string();

    // Apply format-preserving rewrite
    rewrite_preserving_format(&parsed.original_source, &transformed).ok()
}
