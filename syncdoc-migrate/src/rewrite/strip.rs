// syncdoc-migrate/src/rewrite/strip.rs

use crate::extract::{is_inner_doc_attr, is_outer_doc_attr};
use crate::syncdoc_debug;
use proc_macro2::TokenStream;
use quote::quote;
use syncdoc_core::parse::{Attribute, InnerAttribute, ModuleItem};
use unsynn::*;

/// Strips inner doc attributes from module content
pub fn strip_inner_doc_attrs(inner_attrs: &Option<Many<InnerAttribute>>) -> Vec<InnerAttribute> {
    let Some(attr_list) = inner_attrs else {
        return Vec::new();
    };

    attr_list
        .0
        .iter()
        .filter_map(|attr_delimited| {
            if is_inner_doc_attr(&attr_delimited.value) {
                None
            } else {
                Some(attr_delimited.value.clone())
            }
        })
        .collect()
}

/// Strips all doc attributes from a token stream while preserving other attributes
///
/// This is a convenience function that parses the input, strips doc attributes,
/// and returns the modified token stream.
pub fn strip_doc_attrs(item: TokenStream) -> TokenStream {
    // Try to parse as module content
    if let Ok(content) = item
        .clone()
        .into_token_iter()
        .parse::<syncdoc_core::parse::ModuleContent>()
    {
        strip_doc_attrs_from_items(&content)
    } else {
        // If parsing fails, return original
        item
    }
}

/// Updated version that handles both outer and inner attributes
pub fn strip_doc_attrs_from_items(content: &syncdoc_core::parse::ModuleContent) -> TokenStream {
    let mut output = TokenStream::new();

    // Handle inner attributes (module-level docs)
    let stripped_inner = strip_inner_doc_attrs(&content.inner_attrs);
    for attr in stripped_inner {
        quote::ToTokens::to_tokens(&attr, &mut output);
    }

    // Handle items (outer attributes)
    for item_delimited in &content.items.0 {
        let item = &item_delimited.value;
        let processed = strip_doc_attrs_from_item(item);
        output.extend(processed);
    }

    output
}

/// Strip doc attributes from a single item recursively
fn strip_doc_attrs_from_item(item: &ModuleItem) -> TokenStream {
	// DEBUG: Print what kind of item this is
    syncdoc_debug!("Processing item type: {}", match item {
        ModuleItem::Function(_) => "Function",
        ModuleItem::TraitMethod(_) => "TraitMethod",
        ModuleItem::Enum(_) => "Enum",
        ModuleItem::Struct(_) => "Struct",
        ModuleItem::Module(_) => "Module",
        ModuleItem::ImplBlock(_) => "ImplBlock",
        ModuleItem::Trait(_) => "Trait",
        ModuleItem::TypeAlias(_) => "TypeAlias",
        ModuleItem::Const(_) => "Const",
        ModuleItem::Static(_) => "Static",
        ModuleItem::Other(_) => "Other",
    });
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

		ModuleItem::TraitMethod(method) => {
            let stripped_attrs = strip_doc_attrs_from_attr_list(&method.attributes);
            let mut output = TokenStream::new();

            // Add non-doc attributes
            for attr in stripped_attrs {
                quote::ToTokens::to_tokens(&attr, &mut output);
            }

            // Add rest of method signature
            if let Some(const_kw) = &method.const_kw {
                unsynn::ToTokens::to_tokens(const_kw, &mut output);
            }
            if let Some(async_kw) = &method.async_kw {
                unsynn::ToTokens::to_tokens(async_kw, &mut output);
            }
            if let Some(unsafe_kw) = &method.unsafe_kw {
                unsynn::ToTokens::to_tokens(unsafe_kw, &mut output);
            }
            if let Some(extern_kw) = &method.extern_kw {
                unsynn::ToTokens::to_tokens(extern_kw, &mut output);
            }
            unsynn::ToTokens::to_tokens(&method._fn, &mut output);
            quote::ToTokens::to_tokens(&method.name, &mut output);
            if let Some(generics) = &method.generics {
                unsynn::ToTokens::to_tokens(generics, &mut output);
            }
            unsynn::ToTokens::to_tokens(&method.params, &mut output);
            if let Some(ret_type) = &method.return_type {
                unsynn::ToTokens::to_tokens(ret_type, &mut output);
            }
            if let Some(where_clause) = &method.where_clause {
                unsynn::ToTokens::to_tokens(where_clause, &mut output);
            }
            unsynn::ToTokens::to_tokens(&method._semi, &mut output);

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
            let body_stream = extract_brace_group_containing_content(&enum_sig.variants);
            if let Ok(variants) = body_stream
                .into_token_iter()
                .parse::<CommaDelimitedVec<syncdoc_core::parse::EnumVariant>>()
            {
                let processed_variants = strip_doc_attrs_from_variants(&variants);
                output.extend(wrap_in_braces(processed_variants));
            } else {
                unsynn::ToTokens::to_tokens(&enum_sig.variants, &mut output);
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
                    let body_stream = extract_brace_group_containing_content(brace);
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
            let body_stream = extract_brace_group_containing_content(&module.items);
            if let Ok(content) = body_stream
                .into_token_iter()
                .parse::<syncdoc_core::parse::ModuleContent>()
            {
                let processed = strip_doc_attrs_from_items(&content);
                output.extend(wrap_in_braces(processed));
            } else {
                unsynn::ToTokens::to_tokens(&module.items, &mut output);
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
            let body_stream = extract_brace_group_containing_content(&impl_block.items);
            if let Ok(content) = body_stream
                .into_token_iter()
                .parse::<syncdoc_core::parse::ModuleContent>()
            {
                let processed = strip_doc_attrs_from_items(&content);
                output.extend(wrap_in_braces(processed));
            } else {
                unsynn::ToTokens::to_tokens(&impl_block.items, &mut output);
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
            let body_stream = extract_brace_group_containing_content(&trait_def.items);
            if let Ok(content) = body_stream
                .into_token_iter()
                .parse::<syncdoc_core::parse::ModuleContent>()
            {
                let processed = strip_doc_attrs_from_items(&content);
                output.extend(wrap_in_braces(processed));
            } else {
                unsynn::ToTokens::to_tokens(&trait_def.items, &mut output);
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

        // Strip attributes from the variant itself
        let mut variant_output = TokenStream::new();
        quote::ToTokens::to_tokens(&variant.name, &mut variant_output);

        // Handle variant data
        if let Some(data) = &variant.data {
            match data {
                syncdoc_core::parse::EnumVariantData::Struct(fields_containing) => {
                    // NEW: Handle struct variant fields
                    if let Some(fields_cdv) = fields_containing.content.as_ref() {
                        let processed_fields = strip_doc_attrs_from_fields(fields_cdv);
                        variant_output.extend(wrap_in_braces(processed_fields));
                    } else {
                        unsynn::ToTokens::to_tokens(fields_containing, &mut variant_output);
                    }
                }
                _ => {
                    quote::ToTokens::to_tokens(data, &mut variant_output);
                }
            }
        }

        output.extend(variant_output);

        if idx < variants.0.len() - 1 {
            output.extend(quote::quote! { , });
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

fn extract_brace_group_containing_content<T: unsynn::ToTokens>(
    brace_group_containing: &BraceGroupContaining<T>,
) -> TokenStream {
    let mut ts = TokenStream::new();
    unsynn::ToTokens::to_tokens(&brace_group_containing.content, &mut ts);
    ts
}

fn wrap_in_braces(content: TokenStream) -> TokenStream {
    let group = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, content);
    std::iter::once(proc_macro2::TokenTree::Group(group)).collect()
}

#[cfg(test)]
mod strip_tests;
