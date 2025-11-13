// syncdoc-migrate/src/rewrite.rs

use crate::discover::ParsedFile;
use crate::extract::is_outer_doc_attr;
use proc_macro2::TokenStream;
use quote::quote;
use syncdoc_core::parse::{Attribute, ModuleItem};
use unsynn::*;

/// Strips all doc attributes from a token stream while preserving other attributes
///
/// This is a convenience function that parses the input, strips doc attributes,
/// and returns the modified token stream.
pub fn strip_doc_attrs(item: TokenStream) -> TokenStream {
    // Try to parse as module content
    if let Ok(content) = item.clone().into_token_iter().parse::<syncdoc_core::parse::ModuleContent>() {
        strip_doc_attrs_from_items(&content)
    } else {
        // If parsing fails, return original
        item
    }
}

/// Strips all outer doc attributes from parsed items
/// NEVER strips inner attributes (#![...]) as they document the containing module
pub fn strip_doc_attrs_from_items(content: &syncdoc_core::parse::ModuleContent) -> TokenStream {
    let mut output = TokenStream::new();

    for item_delimited in &content.items.0 {
        let item = &item_delimited.value;
        let processed = strip_doc_attrs_from_item(item);
        output.extend(processed);
    }

    output
}

/// Strip doc attributes from a single item recursively
fn strip_doc_attrs_from_item(item: &ModuleItem) -> TokenStream {
    match item {
        ModuleItem::Function(func) => {
            let stripped_attrs = strip_doc_attrs_from_attr_list(&func.attributes);
            let mut output = TokenStream::new();

            // Add non-doc attributes
            for attr in stripped_attrs {
                quote::ToTokens::to_tokens(&attr, &mut output);
            }

            // Add rest of function
            if let Some(vis) = &func.visibility {
                quote::ToTokens::to_tokens(vis, &mut output);
            }
            if let Some(const_kw) = &func.const_kw {
                unsynn::ToTokens::to_tokens(const_kw, &mut output);
            }
            if let Some(async_kw) = &func.async_kw {
                unsynn::ToTokens::to_tokens(async_kw, &mut output);
            }
            if let Some(unsafe_kw) = &func.unsafe_kw {
                unsynn::ToTokens::to_tokens(unsafe_kw, &mut output);
            }
            if let Some(extern_kw) = &func.extern_kw {
                unsynn::ToTokens::to_tokens(extern_kw, &mut output);
            }
            unsynn::ToTokens::to_tokens(&func._fn, &mut output);
            quote::ToTokens::to_tokens(&func.name, &mut output);
            if let Some(generics) = &func.generics {
                unsynn::ToTokens::to_tokens(generics, &mut output);
            }
            unsynn::ToTokens::to_tokens(&func.params, &mut output);
            if let Some(ret_type) = &func.return_type {
                unsynn::ToTokens::to_tokens(ret_type, &mut output);
            }
            if let Some(where_clause) = &func.where_clause {
                unsynn::ToTokens::to_tokens(where_clause, &mut output);
            }
            unsynn::ToTokens::to_tokens(&func.body, &mut output);

            output
        }

        ModuleItem::Enum(enum_sig) => {
            let mut output = TokenStream::new();

            // Strip attributes
            let stripped_attrs = strip_doc_attrs_from_attr_list(&enum_sig.attributes);
            for attr in stripped_attrs {
                quote::ToTokens::to_tokens(&attr, &mut output);
            }

            if let Some(vis) = &enum_sig.visibility {
                quote::ToTokens::to_tokens(vis, &mut output);
            }
            unsynn::ToTokens::to_tokens(&enum_sig._enum, &mut output);
            quote::ToTokens::to_tokens(&enum_sig.name, &mut output);
            if let Some(generics) = &enum_sig.generics {
                unsynn::ToTokens::to_tokens(generics, &mut output);
            }
            if let Some(where_clause) = &enum_sig.where_clause {
                unsynn::ToTokens::to_tokens(where_clause, &mut output);
            }

            // Process enum body - strip docs from variants
            let body_stream = extract_brace_content(&enum_sig.body);
            if let Ok(variants) = body_stream
                .into_token_iter()
                .parse::<CommaDelimitedVec<syncdoc_core::parse::EnumVariant>>()
            {
                let processed_variants = strip_doc_attrs_from_variants(&variants);
                output.extend(wrap_in_braces(processed_variants));
            } else {
                unsynn::ToTokens::to_tokens(&enum_sig.body, &mut output);
            }

            output
        }

        ModuleItem::Struct(struct_sig) => {
            let mut output = TokenStream::new();

            let stripped_attrs = strip_doc_attrs_from_attr_list(&struct_sig.attributes);
            for attr in stripped_attrs {
                quote::ToTokens::to_tokens(&attr, &mut output);
            }

            if let Some(vis) = &struct_sig.visibility {
                quote::ToTokens::to_tokens(vis, &mut output);
            }
            unsynn::ToTokens::to_tokens(&struct_sig._struct, &mut output);
            quote::ToTokens::to_tokens(&struct_sig.name, &mut output);
            if let Some(generics) = &struct_sig.generics {
                unsynn::ToTokens::to_tokens(generics, &mut output);
            }
            if let Some(where_clause) = &struct_sig.where_clause {
                unsynn::ToTokens::to_tokens(where_clause, &mut output);
            }

            // Process struct body - strip docs from fields
            match &struct_sig.body {
                syncdoc_core::parse::StructBody::Named(brace) => {
                    let body_stream = extract_brace_content(brace);
                    if let Ok(fields) = body_stream
                        .into_token_iter()
                        .parse::<CommaDelimitedVec<syncdoc_core::parse::StructField>>()
                    {
                        let processed_fields = strip_doc_attrs_from_fields(&fields);
                        output.extend(wrap_in_braces(processed_fields));
                    } else {
                        unsynn::ToTokens::to_tokens(brace, &mut output);
                    }
                }
                syncdoc_core::parse::StructBody::Tuple(tuple) => {
                    unsynn::ToTokens::to_tokens(tuple, &mut output);
                }
                syncdoc_core::parse::StructBody::Unit(semi) => {
                    unsynn::ToTokens::to_tokens(semi, &mut output);
                }
            }

            output
        }

        ModuleItem::Module(module) => {
            let mut output = TokenStream::new();

            let stripped_attrs = strip_doc_attrs_from_attr_list(&module.attributes);
            for attr in stripped_attrs {
                quote::ToTokens::to_tokens(&attr, &mut output);
            }

            if let Some(vis) = &module.visibility {
                quote::ToTokens::to_tokens(vis, &mut output);
            }
            unsynn::ToTokens::to_tokens(&module._mod, &mut output);
            quote::ToTokens::to_tokens(&module.name, &mut output);

            // Recursively process module body
            let body_stream = extract_brace_content(&module.body);
            if let Ok(content) = body_stream
                .into_token_iter()
                .parse::<syncdoc_core::parse::ModuleContent>()
            {
                let processed = strip_doc_attrs_from_items(&content);
                output.extend(wrap_in_braces(processed));
            } else {
                unsynn::ToTokens::to_tokens(&module.body, &mut output);
            }

            output
        }

        ModuleItem::ImplBlock(impl_block) => {
            let mut output = TokenStream::new();

            let stripped_attrs = strip_doc_attrs_from_attr_list(&impl_block.attributes);
            for attr in stripped_attrs {
                quote::ToTokens::to_tokens(&attr, &mut output);
            }

            unsynn::ToTokens::to_tokens(&impl_block._impl, &mut output);
            if let Some(generics) = &impl_block.generics {
                unsynn::ToTokens::to_tokens(generics, &mut output);
            }
            unsynn::ToTokens::to_tokens(&impl_block.target_type, &mut output);
            if let Some(for_trait) = &impl_block.for_trait {
                unsynn::ToTokens::to_tokens(for_trait, &mut output);
            }
            if let Some(where_clause) = &impl_block.where_clause {
                unsynn::ToTokens::to_tokens(where_clause, &mut output);
            }

            // Recursively process impl body
            let body_stream = extract_brace_content(&impl_block.body);
            if let Ok(content) = body_stream
                .into_token_iter()
                .parse::<syncdoc_core::parse::ModuleContent>()
            {
                let processed = strip_doc_attrs_from_items(&content);
                output.extend(wrap_in_braces(processed));
            } else {
                unsynn::ToTokens::to_tokens(&impl_block.body, &mut output);
            }

            output
        }

        ModuleItem::Trait(trait_def) => {
            let mut output = TokenStream::new();

            let stripped_attrs = strip_doc_attrs_from_attr_list(&trait_def.attributes);
            for attr in stripped_attrs {
                quote::ToTokens::to_tokens(&attr, &mut output);
            }

            if let Some(vis) = &trait_def.visibility {
                quote::ToTokens::to_tokens(vis, &mut output);
            }
            if let Some(unsafe_kw) = &trait_def.unsafe_kw {
                unsynn::ToTokens::to_tokens(unsafe_kw, &mut output);
            }
            unsynn::ToTokens::to_tokens(&trait_def._trait, &mut output);
            quote::ToTokens::to_tokens(&trait_def.name, &mut output);
            if let Some(generics) = &trait_def.generics {
                unsynn::ToTokens::to_tokens(generics, &mut output);
            }
            if let Some(bounds) = &trait_def.bounds {
                unsynn::ToTokens::to_tokens(bounds, &mut output);
            }
            if let Some(where_clause) = &trait_def.where_clause {
                unsynn::ToTokens::to_tokens(where_clause, &mut output);
            }

            // Recursively process trait body
            let body_stream = extract_brace_content(&trait_def.body);
            if let Ok(content) = body_stream
                .into_token_iter()
                .parse::<syncdoc_core::parse::ModuleContent>()
            {
                let processed = strip_doc_attrs_from_items(&content);
                output.extend(wrap_in_braces(processed));
            } else {
                unsynn::ToTokens::to_tokens(&trait_def.body, &mut output);
            }

            output
        }

        ModuleItem::TypeAlias(type_alias) => {
            let mut output = TokenStream::new();

            let stripped_attrs = strip_doc_attrs_from_attr_list(&type_alias.attributes);
            for attr in stripped_attrs {
                quote::ToTokens::to_tokens(&attr, &mut output);
            }

            if let Some(vis) = &type_alias.visibility {
                quote::ToTokens::to_tokens(vis, &mut output);
            }
            unsynn::ToTokens::to_tokens(&type_alias._type, &mut output);
            quote::ToTokens::to_tokens(&type_alias.name, &mut output);
            if let Some(generics) = &type_alias.generics {
                unsynn::ToTokens::to_tokens(generics, &mut output);
            }
            unsynn::ToTokens::to_tokens(&type_alias._eq, &mut output);
            unsynn::ToTokens::to_tokens(&type_alias.target, &mut output);
            unsynn::ToTokens::to_tokens(&type_alias._semi, &mut output);

            output
        }

        ModuleItem::Const(const_sig) => {
            let mut output = TokenStream::new();

            let stripped_attrs = strip_doc_attrs_from_attr_list(&const_sig.attributes);
            for attr in stripped_attrs {
                quote::ToTokens::to_tokens(&attr, &mut output);
            }

            if let Some(vis) = &const_sig.visibility {
                quote::ToTokens::to_tokens(vis, &mut output);
            }
            unsynn::ToTokens::to_tokens(&const_sig._const, &mut output);
            quote::ToTokens::to_tokens(&const_sig.name, &mut output);
            unsynn::ToTokens::to_tokens(&const_sig._colon, &mut output);
            unsynn::ToTokens::to_tokens(&const_sig.const_type, &mut output);
            unsynn::ToTokens::to_tokens(&const_sig._eq, &mut output);
            unsynn::ToTokens::to_tokens(&const_sig.value, &mut output);
            unsynn::ToTokens::to_tokens(&const_sig._semi, &mut output);

            output
        }

        ModuleItem::Static(static_sig) => {
            let mut output = TokenStream::new();

            let stripped_attrs = strip_doc_attrs_from_attr_list(&static_sig.attributes);
            for attr in stripped_attrs {
                quote::ToTokens::to_tokens(&attr, &mut output);
            }

            if let Some(vis) = &static_sig.visibility {
                quote::ToTokens::to_tokens(vis, &mut output);
            }
            if let Some(mut_kw) = &static_sig.mut_kw {
                unsynn::ToTokens::to_tokens(mut_kw, &mut output);
            }
            unsynn::ToTokens::to_tokens(&static_sig._static, &mut output);
            quote::ToTokens::to_tokens(&static_sig.name, &mut output);
            unsynn::ToTokens::to_tokens(&static_sig._colon, &mut output);
            unsynn::ToTokens::to_tokens(&static_sig.static_type, &mut output);
            unsynn::ToTokens::to_tokens(&static_sig._eq, &mut output);
            unsynn::ToTokens::to_tokens(&static_sig.value, &mut output);
            unsynn::ToTokens::to_tokens(&static_sig._semi, &mut output);

            output
        }

        ModuleItem::Other(token) => {
            let mut output = TokenStream::new();
            unsynn::ToTokens::to_tokens(token, &mut output);
            output
        }
    }
}

/// Filter out doc attributes from an attribute list
fn strip_doc_attrs_from_attr_list(attrs: &Option<unsynn::Many<Attribute>>) -> Vec<Attribute> {
    let Some(attr_list) = attrs else {
        return Vec::new();
    };

    attr_list
        .0
        .iter()
        .filter_map(|attr_delimited| {
            let attr = &attr_delimited.value;
            if is_outer_doc_attr(attr) {
                None // Filter out doc attributes
            } else {
                Some(attr.clone()) // Keep non-doc attributes
            }
        })
        .collect()
}

fn strip_doc_attrs_from_variants(
    variants: &CommaDelimitedVec<syncdoc_core::parse::EnumVariant>,
) -> TokenStream {
    let mut output = TokenStream::new();

    for (idx, variant_delimited) in variants.0.iter().enumerate() {
        let variant = &variant_delimited.value;

        // Strip variant attributes
        let stripped_attrs = strip_doc_attrs_from_attr_list(&variant.attributes);
        for attr in stripped_attrs {
            quote::ToTokens::to_tokens(&attr, &mut output);
        }

        quote::ToTokens::to_tokens(&variant.name, &mut output);
        if let Some(data) = &variant.data {
            quote::ToTokens::to_tokens(data, &mut output);
        }

        if idx < variants.0.len() - 1 {
            output.extend(quote! { , });
        }
    }

    output
}

fn strip_doc_attrs_from_fields(
    fields: &CommaDelimitedVec<syncdoc_core::parse::StructField>,
) -> TokenStream {
    let mut output = TokenStream::new();

    for (idx, field_delimited) in fields.0.iter().enumerate() {
        let field = &field_delimited.value;

        // Strip field attributes
        let stripped_attrs = strip_doc_attrs_from_attr_list(&field.attributes);
        for attr in stripped_attrs {
            quote::ToTokens::to_tokens(&attr, &mut output);
        }

        if let Some(vis) = &field.visibility {
            quote::ToTokens::to_tokens(vis, &mut output);
        }
        quote::ToTokens::to_tokens(&field.name, &mut output);
        unsynn::ToTokens::to_tokens(&field._colon, &mut output);
        unsynn::ToTokens::to_tokens(&field.field_type, &mut output);

        if idx < fields.0.len() - 1 {
            output.extend(quote! { , });
        }
    }

    output
}

fn extract_brace_content(brace_group: &unsynn::BraceGroup) -> TokenStream {
    let mut ts = TokenStream::new();
    unsynn::ToTokens::to_tokens(brace_group, &mut ts);
    if let Some(proc_macro2::TokenTree::Group(g)) = ts.into_iter().next() {
        g.stream()
    } else {
        TokenStream::new()
    }
}

fn wrap_in_braces(content: TokenStream) -> TokenStream {
    let group = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, content);
    std::iter::once(proc_macro2::TokenTree::Group(group)).collect()
}

/// Injects `#[omnidoc(path = "...")]` attribute into an item's token stream
pub fn inject_omnidoc_attr(item: TokenStream, docs_root: &str) -> TokenStream {
    let mut output = TokenStream::new();

    // Parse to find where to inject (after attributes, before visibility/keywords)
    if let Ok(content) = item
        .clone()
        .into_token_iter()
        .parse::<syncdoc_core::parse::ModuleContent>()
    {
        if let Some(first_item) = content.items.0.first() {
            // Get attributes from the parsed item
            let attrs_opt = match &first_item.value {
                ModuleItem::Function(f) => &f.attributes,
                ModuleItem::Enum(e) => &e.attributes,
                ModuleItem::Struct(s) => &s.attributes,
                ModuleItem::Module(m) => &m.attributes,
                ModuleItem::Trait(t) => &t.attributes,
                ModuleItem::ImplBlock(i) => &i.attributes,
                ModuleItem::TypeAlias(ta) => &ta.attributes,
                ModuleItem::Const(c) => &c.attributes,
                ModuleItem::Static(s) => &s.attributes,
                _ => &None,
            };

            // Emit existing attributes
            if let Some(attrs) = attrs_opt {
                for attr in &attrs.0 {
                    quote::ToTokens::to_tokens(&attr.value, &mut output);
                }
            }

            // Inject omnidoc attribute
            let attr = quote! {
                #[syncdoc::omnidoc(path = #docs_root)]
            };
            output.extend(attr);

            // Emit the rest of the item without its attributes
            // (this is complex - for now just re-emit the whole thing)
            // TODO: proper reconstruction without attributes
            output.extend(item);
            return output;
        }
    }

    // Fallback: inject at the beginning
    let attr = quote! {
        #[syncdoc::omnidoc(path = #docs_root)]
    };
    output.extend(attr);
    output.extend(item);
    output
}

/// Rewrites a parsed file by stripping doc attrs and/or injecting omnidoc attributes
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
        strip_doc_attrs_from_items(&parsed.content)
    } else {
        let mut ts = TokenStream::new();
        quote::ToTokens::to_tokens(&parsed.content, &mut ts);
        ts
    };

    if annotate {
        // Re-parse and inject omnidoc into each item
        if let Ok(content) = output
            .clone()
            .into_token_iter()
            .parse::<syncdoc_core::parse::ModuleContent>()
        {
            let mut annotated = TokenStream::new();
            for item_delimited in &content.items.0 {
                let mut item_ts = TokenStream::new();
                quote::ToTokens::to_tokens(&item_delimited.value, &mut item_ts);

                // Only annotate named items
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
mod tests;
