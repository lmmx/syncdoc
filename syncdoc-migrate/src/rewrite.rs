// syncdoc-migrate/src/rewrite.rs

use crate::discover::ParsedFile;
use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syncdoc_core::parse::ModuleItem;

/// Strips all doc attributes from a token stream while preserving other attributes
pub fn strip_doc_attrs(item: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();
    let tokens: Vec<TokenTree> = item.into_iter().collect();
    let mut i = 0;

    while i < tokens.len() {
        // Check if we're at the start of an attribute
        if matches!(tokens.get(i), Some(TokenTree::Punct(p)) if p.as_char() == '#') {
            // Look ahead for the bracket
            if matches!(tokens.get(i + 1), Some(TokenTree::Group(g)) if g.delimiter() == proc_macro2::Delimiter::Bracket)
            {
                if let Some(TokenTree::Group(attr_group)) = tokens.get(i + 1) {
                    // Check if this is a doc attribute
                    if is_doc_attribute(attr_group.stream()) {
                        // Skip both the # and the [...] group
                        i += 2;
                        continue;
                    }
                }
            }
        }

        // Not a doc attribute, preserve the token
        output.extend(std::iter::once(tokens[i].clone()));
        i += 1;
    }

    output
}

/// Recursively strips doc attributes from nested items
fn strip_doc_attrs_recursive(item: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();
    let tokens: Vec<TokenTree> = item.into_iter().collect();
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            // Handle attributes
            TokenTree::Punct(p) if p.as_char() == '#' => {
                if matches!(tokens.get(i + 1), Some(TokenTree::Group(g)) if g.delimiter() == proc_macro2::Delimiter::Bracket)
                {
                    if let Some(TokenTree::Group(attr_group)) = tokens.get(i + 1) {
                        if is_doc_attribute(attr_group.stream()) {
                            // Skip doc attribute
                            i += 2;
                            continue;
                        }
                    }
                }
                output.extend(std::iter::once(tokens[i].clone()));
                i += 1;
            }
            // Recursively handle groups (braces, brackets, parens)
            TokenTree::Group(g) => {
                let stripped_inner = strip_doc_attrs_recursive(g.stream());
                let new_group = proc_macro2::Group::new(g.delimiter(), stripped_inner);
                output.extend(std::iter::once(TokenTree::Group(new_group)));
                i += 1;
            }
            // Pass through other tokens
            _ => {
                output.extend(std::iter::once(tokens[i].clone()));
                i += 1;
            }
        }
    }

    output
}

/// Checks if an attribute group contains "doc"
fn is_doc_attribute(attr_content: TokenStream) -> bool {
    let attr_str = attr_content.to_string();

    // Check for various doc attribute patterns:
    // - doc = "..."
    // - doc(hidden)
    // - cfg_attr(doc, ...)

    // Simple check: starts with "doc" (possibly with whitespace)
    let trimmed = attr_str.trim_start();

    // Match "doc" as the first identifier
    if trimmed.starts_with("doc") {
        // Make sure it's followed by = or ( or whitespace, not part of another word
        if let Some(next_char) = trimmed.chars().nth(3) {
            matches!(next_char, '=' | '(' | ' ' | '\t' | '\n')
        } else {
            // "doc" at the end
            true
        }
    } else if trimmed.starts_with("cfg_attr") {
        // Check if cfg_attr contains doc
        attr_str.contains("doc")
    } else {
        false
    }
}

/// Injects `#[omnidoc(path = "...")]` attribute into an item's token stream
///
/// The path should be the docs root directory, not the specific file.
/// omnidoc automatically finds the right file based on the item name.
///
/// Places the attribute after visibility modifiers but before other attributes.
pub fn inject_omnidoc_attr(item: TokenStream, docs_root: &str) -> TokenStream {
    let tokens: Vec<TokenTree> = item.into_iter().collect();
    let mut output = TokenStream::new();
    let mut vis_end_idx = 0;

    // Find the end of visibility modifiers
    let mut i = 0;
    while i < tokens.len() {
        if let Some(TokenTree::Ident(ident)) = tokens.get(i) {
            if *ident == "pub" {
                vis_end_idx = i + 1;

                // Check for pub(crate), pub(super), etc.
                if let Some(TokenTree::Group(g)) = tokens.get(i + 1) {
                    if g.delimiter() == proc_macro2::Delimiter::Parenthesis {
                        vis_end_idx = i + 2;
                    }
                }
                break;
            }
        }

        // If we hit an attribute or keyword, we're past visibility
        if matches!(tokens.get(i), Some(TokenTree::Punct(p)) if p.as_char() == '#') {
            break;
        }
        if let Some(TokenTree::Ident(ident)) = tokens.get(i) {
            let ident_str = ident.to_string();
            if matches!(
                ident_str.as_str(),
                "fn" | "struct"
                    | "enum"
                    | "trait"
                    | "impl"
                    | "mod"
                    | "const"
                    | "static"
                    | "type"
                    | "async"
                    | "unsafe"
                    | "extern"
            ) {
                break;
            }
        }

        i += 1;
    }

    // Add tokens up to visibility end
    output.extend(tokens[..vis_end_idx].iter().cloned());

    // Add the omnidoc attribute with docs root
    let attr = quote! {
        #[omnidoc(path = #docs_root)]
    };
    output.extend(attr);

    // Add the rest of the tokens
    output.extend(tokens[vis_end_idx..].iter().cloned());

    output
}

/// Rewrites a parsed file by stripping doc attrs and/or injecting omnidoc attributes
///
/// Returns `None` if neither strip nor annotate is requested (no rewrite needed).
/// Returns `Some(String)` with the rewritten source code otherwise.
///
/// **All items get `#[omnidoc(path = docs_root)]`**, which:
/// - For containers (modules, impls, traits, structs with fields, enums with variants):
///   automatically documents the container and all its children
/// - For leaf items (functions, type aliases, etc.): acts like syncdoc and documents just that item
pub fn rewrite_file(
    parsed: &ParsedFile,
    docs_root: &str,
    strip: bool,
    annotate: bool,
) -> Option<String> {
    if !strip && !annotate {
        return None;
    }

    let mut output = TokenStream::new();

    for item_delimited in &parsed.content.items.0 {
        let item = &item_delimited.value;
        let mut item_tokens = TokenStream::new();
        unsynn::ToTokens::to_tokens(item, &mut item_tokens);

        // Apply strip if requested (recursively strips from nested items too)
        if strip {
            item_tokens = strip_doc_attrs_recursive(item_tokens);
        }

        // Apply annotation if requested
        // All items get omnidoc with the docs root path
        if annotate {
            // Only annotate items that have names (skip Other variants)
            if get_item_name(item).is_some() {
                item_tokens = inject_omnidoc_attr(item_tokens, docs_root);
            }
        }

        output.extend(item_tokens);
    }

    Some(output.to_string())
}

/// Extracts the name from a module item
fn get_item_name(item: &ModuleItem) -> Option<String> {
    match item {
        ModuleItem::Function(f) => Some(f.name.to_string()),
        ModuleItem::Module(m) => Some(m.name.to_string()),
        ModuleItem::Trait(t) => Some(t.name.to_string()),
        ModuleItem::Enum(e) => Some(e.name.to_string()),
        ModuleItem::Struct(s) => Some(s.name.to_string()),
        ModuleItem::TypeAlias(ta) => Some(ta.name.to_string()),
        ModuleItem::Const(c) => Some(c.name.to_string()),
        ModuleItem::Static(s) => Some(s.name.to_string()),
        ModuleItem::ImplBlock(impl_block) => {
            // Extract type name from target_type
            if let Some(first) = impl_block.target_type.0.first() {
                if let proc_macro2::TokenTree::Ident(ident) = &first.value.second {
                    return Some(ident.to_string());
                }
            }
            None
        }
        ModuleItem::Other(_) => None,
    }
}

#[cfg(test)]
mod tests;
