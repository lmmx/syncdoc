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

/// Determines if an item should receive `#[omnidoc]` vs `#[syncdoc]`
///
/// Returns true for items that contain multiple documentable children:
/// - Modules (always have potential children)
/// - Impl blocks (always have potential methods)
/// - Traits (always have potential methods)
/// - Enums (if they have variants)
/// - Structs (if they have named fields)
///
/// These items get `#[omnidoc]` on the parent, and their children get NO annotation
/// (because omnidoc automatically handles them).
pub fn needs_omnidoc(item: &ModuleItem) -> bool {
    match item {
        ModuleItem::Module(_) => true,
        ModuleItem::ImplBlock(_) => true,
        ModuleItem::Trait(_) => true,
        ModuleItem::Enum(enum_sig) => {
            // Check if enum has variants
            has_enum_variants(&enum_sig.body)
        }
        ModuleItem::Struct(struct_sig) => {
            // Only named structs with fields get omnidoc
            matches!(struct_sig.body, syncdoc_core::parse::StructBody::Named(_))
                && has_struct_fields(&struct_sig.body)
        }
        ModuleItem::Function(_) => false,
        ModuleItem::TypeAlias(_) => false,
        ModuleItem::Const(_) => false,
        ModuleItem::Static(_) => false,
        ModuleItem::Other(_) => false,
    }
}

fn has_enum_variants(body: &unsynn::BraceGroup) -> bool {
    let mut ts = TokenStream::new();
    unsynn::ToTokens::to_tokens(body, &mut ts);

    if let Some(TokenTree::Group(g)) = ts.into_iter().next() {
        let content = g.stream().to_string();
        // Simple heuristic: if there's content and it's not just whitespace
        !content.trim().is_empty()
    } else {
        false
    }
}

fn has_struct_fields(body: &syncdoc_core::parse::StructBody) -> bool {
    if let syncdoc_core::parse::StructBody::Named(brace_group) = body {
        let mut ts = TokenStream::new();
        unsynn::ToTokens::to_tokens(brace_group, &mut ts);

        if let Some(TokenTree::Group(g)) = ts.into_iter().next() {
            let content = g.stream().to_string();
            !content.trim().is_empty()
        } else {
            false
        }
    } else {
        false
    }
}

/// Injects `#[syncdoc(path = "...")]` attribute into an item's token stream
///
/// Places the attribute after visibility modifiers but before other attributes
pub fn inject_syncdoc_attr(item: TokenStream, doc_path: &str) -> TokenStream {
    inject_attr_after_visibility(item, doc_path, "syncdoc")
}

/// Injects `#[omnidoc(path = "...")]` attribute into an item's token stream
///
/// Places the attribute after visibility modifiers but before other attributes
pub fn inject_omnidoc_attr(item: TokenStream, doc_path: &str) -> TokenStream {
    inject_attr_after_visibility(item, doc_path, "omnidoc")
}

fn inject_attr_after_visibility(item: TokenStream, doc_path: &str, attr_name: &str) -> TokenStream {
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

    // Add the syncdoc/omnidoc attribute
    let attr_ident = proc_macro2::Ident::new(attr_name, proc_macro2::Span::call_site());
    let attr = quote! {
        #[#attr_ident(path = #doc_path)]
    };
    output.extend(attr);

    // Add the rest of the tokens
    output.extend(tokens[vis_end_idx..].iter().cloned());

    output
}

/// Rewrites a parsed file by stripping doc attrs and/or injecting syncdoc/omnidoc attributes
///
/// Returns `None` if neither strip nor annotate is requested (no rewrite needed).
/// Returns `Some(String)` with the rewritten source code otherwise.
///
/// **Important**: If an item gets `#[omnidoc]`, its children do NOT get annotated,
/// because omnidoc automatically handles all nested documentation.
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
        // Only annotate top-level items; omnidoc handles children automatically
        if annotate {
            let item_name = get_item_name(item);
            if let Some(name) = item_name {
                let doc_path = format!("{}/{}.md", docs_root, name);

                if needs_omnidoc(item) {
                    // Parent gets omnidoc, children get nothing
                    item_tokens = inject_omnidoc_attr(item_tokens, &doc_path);
                } else {
                    // Standalone item gets syncdoc
                    item_tokens = inject_syncdoc_attr(item_tokens, &doc_path);
                }
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
