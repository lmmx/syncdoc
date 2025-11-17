//! Restore inline documentation from external markdown files

use crate::discover::ParsedFile;
use crate::extract::is_inner_doc_attr;
use proc_macro2::TokenStream;
use quote::quote;
use std::fs;
use syncdoc_core::parse::*;
use unsynn::*;

/// Restores inline documentation by reading markdown files and converting omnidoc attributes
pub fn restore_file(parsed: &ParsedFile, docs_root: &str) -> Option<String> {
    let mut output = TokenStream::new();

    // Handle module-level docs
    if let Some(inner_attrs) = &parsed.content.inner_attrs {
        let has_module_doc = check_for_module_doc_macro(inner_attrs);

        if has_module_doc {
            // Read the module markdown file
            if let Some(doc_content) = read_module_doc(parsed, docs_root) {
                output.extend(generate_module_doc_comments(&doc_content));
            }
        } else {
            // Keep non-module-doc inner attributes
            for attr in &inner_attrs.0 {
                if !is_inner_doc_attr(&attr.value) {
                    quote::ToTokens::to_tokens(&attr.value, &mut output);
                }
            }
        }
    }

    // Process items
    for item_delimited in &parsed.content.items.0 {
        output.extend(restore_item(
            &item_delimited.value,
            Vec::new(),
            docs_root,
            parsed,
        ));
    }

    Some(output.to_string())
}

fn check_for_module_doc_macro(inner_attrs: &Many<InnerAttribute>) -> bool {
    for attr in &inner_attrs.0 {
        let mut ts = TokenStream::new();
        unsynn::ToTokens::to_tokens(&attr.value, &mut ts);
        let s = ts.to_string().replace(' ', "");
        if s.contains("module_doc!") {
            return true;
        }
    }
    false
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

    fs::read_to_string(&md_path).ok()
}

fn generate_module_doc_comments(content: &str) -> TokenStream {
    let lines: Vec<_> = content.trim_end().lines().collect();
    let mut output = TokenStream::new();

    for line in lines {
        let comment = format!("//! {}", line);
        output.extend(quote! { #[doc = #comment] });
    }

    output
}

fn restore_item(
    item: &ModuleItem,
    context: Vec<String>,
    docs_root: &str,
    parsed: &ParsedFile,
) -> TokenStream {
    match item {
        ModuleItem::Function(func) => restore_function(func, &context, docs_root),
        ModuleItem::Struct(s) => restore_struct(s, &context, docs_root),
        ModuleItem::Enum(e) => restore_enum(e, &context, docs_root),
        ModuleItem::Module(m) => restore_module(m, context, docs_root, parsed),
        ModuleItem::Trait(t) => restore_trait(t, context, docs_root, parsed),
        ModuleItem::ImplBlock(i) => restore_impl(i, context, docs_root, parsed),
        ModuleItem::TypeAlias(ta) => restore_type_alias(ta, &context, docs_root),
        ModuleItem::Const(c) => restore_const(c, &context, docs_root),
        ModuleItem::Static(s) => restore_static(s, &context, docs_root),
        ModuleItem::TraitMethod(tm) => restore_trait_method(tm, &context, docs_root),
        ModuleItem::Other(t) => {
            let mut ts = TokenStream::new();
            unsynn::ToTokens::to_tokens(t, &mut ts);
            ts
        }
    }
}

fn restore_function(func: &FnSig, context: &[String], docs_root: &str) -> TokenStream {
    let mut output = TokenStream::new();

    // Read markdown file
    let md_content = read_item_markdown(context, &func.name.to_string(), docs_root);

    // Add doc comments
    if let Some(content) = md_content {
        output.extend(generate_doc_comments(&content));
    }

    // Add non-omnidoc attributes
    if let Some(attrs) = &func.attributes {
        for attr in &attrs.0 {
            if !is_omnidoc_attr(&attr.value) {
                quote::ToTokens::to_tokens(&attr.value, &mut output);
            }
        }
    }

    // Add function signature
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
    if let Some(ret) = &func.return_type {
        unsynn::ToTokens::to_tokens(ret, &mut output);
    }
    if let Some(where_clause) = &func.where_clause {
        unsynn::ToTokens::to_tokens(where_clause, &mut output);
    }
    unsynn::ToTokens::to_tokens(&func.body, &mut output);

    output
}

fn is_omnidoc_attr(attr: &Attribute) -> bool {
    let mut ts = TokenStream::new();
    unsynn::ToTokens::to_tokens(attr, &mut ts);
    let s = ts.to_string().replace(' ', "");
    s.contains("omnidoc") || s.contains("syncdoc::omnidoc")
}

fn read_item_markdown(context: &[String], item_name: &str, docs_root: &str) -> Option<String> {
    let mut path_parts = vec![docs_root.to_string()];
    path_parts.extend(context.iter().cloned());
    path_parts.push(format!("{}.md", item_name));

    let md_path = path_parts.join("/");
    fs::read_to_string(&md_path).ok()
}

fn generate_doc_comments(content: &str) -> TokenStream {
    let lines: Vec<_> = content.trim_end().lines().collect();
    let mut output = TokenStream::new();

    for line in lines {
        let comment = format!("/// {}", line);
        output.extend(quote! { #[doc = #comment] });
    }

    output
}

fn restore_struct(struct_sig: &StructSig, context: &[String], docs_root: &str) -> TokenStream {
    let mut output = TokenStream::new();
    let struct_name = struct_sig.name.to_string();

    // Read struct docs
    if let Some(content) = read_item_markdown(context, &struct_name, docs_root) {
        output.extend(generate_doc_comments(&content));
    }

    // Non-omnidoc attributes
    if let Some(attrs) = &struct_sig.attributes {
        for attr in &attrs.0 {
            if !is_omnidoc_attr(&attr.value) {
                quote::ToTokens::to_tokens(&attr.value, &mut output);
            }
        }
    }

    // Struct declaration
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

    // Restore fields with docs
    match &struct_sig.body {
        StructBody::Named(fields) => {
            let restored_fields = restore_struct_fields(fields, &struct_name, context, docs_root);
            output.extend(wrap_in_braces(restored_fields));
        }
        other => {
            quote::ToTokens::to_tokens(other, &mut output);
        }
    }

    output
}

fn restore_struct_fields(
    fields: &BraceGroupContaining<Option<CommaDelimitedVec<StructField>>>,
    struct_name: &str,
    context: &[String],
    docs_root: &str,
) -> TokenStream {
    let mut output = TokenStream::new();

    if let Some(fields_cdv) = fields.content.as_ref() {
        for (idx, field_delimited) in fields_cdv.0.iter().enumerate() {
            let field = &field_delimited.value;
            let field_name = field.name.to_string();

            // Read field docs
            let mut field_context = context.to_vec();
            field_context.push(struct_name.to_string());
            if let Some(content) = read_item_markdown(&field_context, &field_name, docs_root) {
                output.extend(generate_doc_comments(&content));
            }

            // Non-omnidoc attributes
            if let Some(attrs) = &field.attributes {
                for attr in &attrs.0 {
                    if !is_omnidoc_attr(&attr.value) {
                        quote::ToTokens::to_tokens(&attr.value, &mut output);
                    }
                }
            }

            // Field declaration
            if let Some(vis) = &field.visibility {
                quote::ToTokens::to_tokens(vis, &mut output);
            }
            quote::ToTokens::to_tokens(&field.name, &mut output);
            unsynn::ToTokens::to_tokens(&field._colon, &mut output);
            unsynn::ToTokens::to_tokens(&field.field_type, &mut output);

            if idx < fields_cdv.0.len() - 1 {
                output.extend(quote! { , });
            }
        }
    }

    output
}

fn restore_enum(enum_sig: &EnumSig, context: &[String], docs_root: &str) -> TokenStream {
    let mut output = TokenStream::new();
    let enum_name = enum_sig.name.to_string();

    // Read enum docs
    if let Some(content) = read_item_markdown(context, &enum_name, docs_root) {
        output.extend(generate_doc_comments(&content));
    }

    // Non-omnidoc attributes
    if let Some(attrs) = &enum_sig.attributes {
        for attr in &attrs.0 {
            if !is_omnidoc_attr(&attr.value) {
                quote::ToTokens::to_tokens(&attr.value, &mut output);
            }
        }
    }

    // Enum declaration
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

    // Restore variants with docs
    let restored_variants =
        restore_enum_variants(&enum_sig.variants, &enum_name, context, docs_root);
    output.extend(wrap_in_braces(restored_variants));

    output
}

fn restore_enum_variants(
    variants: &BraceGroupContaining<Option<CommaDelimitedVec<EnumVariant>>>,
    enum_name: &str,
    context: &[String],
    docs_root: &str,
) -> TokenStream {
    let mut output = TokenStream::new();

    if let Some(variants_cdv) = variants.content.as_ref() {
        for (idx, variant_delimited) in variants_cdv.0.iter().enumerate() {
            let variant = &variant_delimited.value;
            let variant_name = variant.name.to_string();

            // Read variant docs
            let mut variant_context = context.to_vec();
            variant_context.push(enum_name.to_string());
            if let Some(content) = read_item_markdown(&variant_context, &variant_name, docs_root) {
                output.extend(generate_doc_comments(&content));
            }

            // Non-omnidoc attributes
            if let Some(attrs) = &variant.attributes {
                for attr in &attrs.0 {
                    if !is_omnidoc_attr(&attr.value) {
                        quote::ToTokens::to_tokens(&attr.value, &mut output);
                    }
                }
            }

            // Variant name
            quote::ToTokens::to_tokens(&variant.name, &mut output);

            // Variant data (handle struct variants with fields)
            if let Some(data) = &variant.data {
                match data {
                    EnumVariantData::Struct(fields_containing) => {
                        // Restore struct variant fields
                        if let Some(fields_cdv) = fields_containing.content.as_ref() {
                            let restored_fields = restore_enum_variant_fields(
                                fields_cdv,
                                enum_name,
                                &variant_name,
                                context,
                                docs_root,
                            );
                            output.extend(wrap_in_braces(restored_fields));
                        } else {
                            unsynn::ToTokens::to_tokens(fields_containing, &mut output);
                        }
                    }
                    other => {
                        quote::ToTokens::to_tokens(other, &mut output);
                    }
                }
            }

            if idx < variants_cdv.0.len() - 1 {
                output.extend(quote! { , });
            }
        }
    }

    output
}

fn restore_enum_variant_fields(
    fields: &CommaDelimitedVec<StructField>,
    enum_name: &str,
    variant_name: &str,
    context: &[String],
    docs_root: &str,
) -> TokenStream {
    let mut output = TokenStream::new();

    for (idx, field_delimited) in fields.0.iter().enumerate() {
        let field = &field_delimited.value;
        let field_name = field.name.to_string();

        // Read field docs
        let mut field_context = context.to_vec();
        field_context.push(enum_name.to_string());
        field_context.push(variant_name.to_string());
        if let Some(content) = read_item_markdown(&field_context, &field_name, docs_root) {
            output.extend(generate_doc_comments(&content));
        }

        // Non-omnidoc attributes
        if let Some(attrs) = &field.attributes {
            for attr in &attrs.0 {
                if !is_omnidoc_attr(&attr.value) {
                    quote::ToTokens::to_tokens(&attr.value, &mut output);
                }
            }
        }

        // Field declaration
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

fn restore_module(
    module: &ModuleSig,
    context: Vec<String>,
    docs_root: &str,
    parsed: &ParsedFile,
) -> TokenStream {
    let mut output = TokenStream::new();
    let module_name = module.name.to_string();

    // Read module docs
    if let Some(content) = read_item_markdown(&context, &module_name, docs_root) {
        output.extend(generate_doc_comments(&content));
    }

    // Non-omnidoc attributes
    if let Some(attrs) = &module.attributes {
        for attr in &attrs.0 {
            if !is_omnidoc_attr(&attr.value) {
                quote::ToTokens::to_tokens(&attr.value, &mut output);
            }
        }
    }

    // Module declaration
    if let Some(vis) = &module.visibility {
        quote::ToTokens::to_tokens(vis, &mut output);
    }
    unsynn::ToTokens::to_tokens(&module._mod, &mut output);
    quote::ToTokens::to_tokens(&module.name, &mut output);

    // Recursively restore module contents
    let mut new_context = context;
    new_context.push(module_name);

    let mut module_content = TokenStream::new();
    for item_delimited in &module.items.content.items.0 {
        module_content.extend(restore_item(
            &item_delimited.value,
            new_context.clone(),
            docs_root,
            parsed,
        ));
    }

    output.extend(wrap_in_braces(module_content));
    output
}

fn restore_trait(
    trait_def: &TraitSig,
    context: Vec<String>,
    docs_root: &str,
    parsed: &ParsedFile,
) -> TokenStream {
    let mut output = TokenStream::new();
    let trait_name = trait_def.name.to_string();

    // Read trait docs
    if let Some(content) = read_item_markdown(&context, &trait_name, docs_root) {
        output.extend(generate_doc_comments(&content));
    }

    // Non-omnidoc attributes
    if let Some(attrs) = &trait_def.attributes {
        for attr in &attrs.0 {
            if !is_omnidoc_attr(&attr.value) {
                quote::ToTokens::to_tokens(&attr.value, &mut output);
            }
        }
    }

    // Trait declaration
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

    // Recursively restore trait contents
    let mut new_context = context;
    new_context.push(trait_name);

    let mut trait_content = TokenStream::new();
    for item_delimited in &trait_def.items.content.items.0 {
        trait_content.extend(restore_item(
            &item_delimited.value,
            new_context.clone(),
            docs_root,
            parsed,
        ));
    }

    output.extend(wrap_in_braces(trait_content));
    output
}

fn restore_impl(
    impl_block: &ImplBlockSig,
    context: Vec<String>,
    docs_root: &str,
    parsed: &ParsedFile,
) -> TokenStream {
    let mut output = TokenStream::new();

    // Non-omnidoc attributes
    if let Some(attrs) = &impl_block.attributes {
        for attr in &attrs.0 {
            if !is_omnidoc_attr(&attr.value) {
                quote::ToTokens::to_tokens(&attr.value, &mut output);
            }
        }
    }

    // Impl declaration
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

    // Determine context for impl block methods
    let impl_context = if let Some(for_trait) = &impl_block.for_trait {
        // impl Trait for Type
        let trait_name = if let Some(first) = impl_block.target_type.0.first() {
            if let proc_macro2::TokenTree::Ident(ident) = &first.value.second {
                ident.to_string()
            } else {
                "Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
        };

        let type_name = if let Some(first) = for_trait.second.0.first() {
            if let proc_macro2::TokenTree::Ident(ident) = &first.value.second {
                ident.to_string()
            } else {
                "Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
        };

        vec![type_name, trait_name]
    } else {
        // impl Type
        let type_name = if let Some(first) = impl_block.target_type.0.first() {
            if let proc_macro2::TokenTree::Ident(ident) = &first.value.second {
                ident.to_string()
            } else {
                "Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
        };
        vec![type_name]
    };

    let mut new_context = context;
    new_context.extend(impl_context);

    // Recursively restore impl contents
    let mut impl_content = TokenStream::new();
    for item_delimited in &impl_block.items.content.items.0 {
        impl_content.extend(restore_item(
            &item_delimited.value,
            new_context.clone(),
            docs_root,
            parsed,
        ));
    }

    output.extend(wrap_in_braces(impl_content));
    output
}

fn restore_type_alias(
    type_alias: &TypeAliasSig,
    context: &[String],
    docs_root: &str,
) -> TokenStream {
    let mut output = TokenStream::new();

    // Read docs
    if let Some(content) = read_item_markdown(context, &type_alias.name.to_string(), docs_root) {
        output.extend(generate_doc_comments(&content));
    }

    // Non-omnidoc attributes
    if let Some(attrs) = &type_alias.attributes {
        for attr in &attrs.0 {
            if !is_omnidoc_attr(&attr.value) {
                quote::ToTokens::to_tokens(&attr.value, &mut output);
            }
        }
    }

    // Type alias declaration
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

fn restore_const(const_sig: &ConstSig, context: &[String], docs_root: &str) -> TokenStream {
    let mut output = TokenStream::new();

    // Read docs
    if let Some(content) = read_item_markdown(context, &const_sig.name.to_string(), docs_root) {
        output.extend(generate_doc_comments(&content));
    }

    // Non-omnidoc attributes
    if let Some(attrs) = &const_sig.attributes {
        for attr in &attrs.0 {
            if !is_omnidoc_attr(&attr.value) {
                quote::ToTokens::to_tokens(&attr.value, &mut output);
            }
        }
    }

    // Const declaration
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

fn restore_static(static_sig: &StaticSig, context: &[String], docs_root: &str) -> TokenStream {
    let mut output = TokenStream::new();

    // Read docs
    if let Some(content) = read_item_markdown(context, &static_sig.name.to_string(), docs_root) {
        output.extend(generate_doc_comments(&content));
    }

    // Non-omnidoc attributes
    if let Some(attrs) = &static_sig.attributes {
        for attr in &attrs.0 {
            if !is_omnidoc_attr(&attr.value) {
                quote::ToTokens::to_tokens(&attr.value, &mut output);
            }
        }
    }

    // Static declaration
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

fn restore_trait_method(
    method: &TraitMethodSig,
    context: &[String],
    docs_root: &str,
) -> TokenStream {
    let mut output = TokenStream::new();

    // Read docs
    if let Some(content) = read_item_markdown(context, &method.name.to_string(), docs_root) {
        output.extend(generate_doc_comments(&content));
    }

    // Non-omnidoc attributes
    if let Some(attrs) = &method.attributes {
        for attr in &attrs.0 {
            if !is_omnidoc_attr(&attr.value) {
                quote::ToTokens::to_tokens(&attr.value, &mut output);
            }
        }
    }

    // Method signature
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
    if let Some(ret) = &method.return_type {
        unsynn::ToTokens::to_tokens(ret, &mut output);
    }
    if let Some(where_clause) = &method.where_clause {
        unsynn::ToTokens::to_tokens(where_clause, &mut output);
    }
    unsynn::ToTokens::to_tokens(&method._semi, &mut output);

    output
}

fn wrap_in_braces(content: TokenStream) -> TokenStream {
    let group = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, content);
    std::iter::once(proc_macro2::TokenTree::Group(group)).collect()
}

#[cfg(test)]
mod tests;
