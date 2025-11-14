// syncdoc-migrate/src/rewrite.rs

mod inject;
mod strip;

pub use inject::{inject_module_doc_attr, inject_omnidoc_attr};
pub use strip::strip_doc_attrs;
use unsynn::*;

use crate::discover::ParsedFile;
use proc_macro2::TokenStream;
use syncdoc_core::parse::ModuleItem;

pub fn rewrite_file(
    parsed: &ParsedFile,
    docs_root: &str,
    strip: bool,
    annotate: bool,
) -> Option<String> {
    if !strip && !annotate {
        return None;
    }

    let mut output = if strip {
        strip::strip_doc_attrs_from_items(&parsed.content)
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

            // Inject module_doc! for inner docs if any existed
            if content.inner_attrs.is_some() || parsed.content.inner_attrs.is_some() {
                annotated.extend(inject_module_doc_attr(docs_root));
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
                    annotated.extend(inject_omnidoc_attr(item_ts, docs_root));
                } else {
                    annotated.extend(item_ts);
                }
            }
            output = annotated;
        }
    }

    Some(output.to_string())
}

#[cfg(test)]
mod inject_tests;
#[cfg(test)]
mod rewrite_tests;
#[cfg(test)]
mod strip_tests;
