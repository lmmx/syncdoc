//! Documentation injection logic for restore operation
//!
//! Injects inline doc comments by reading from markdown files, removing omnidoc attributes.

use crate::discover::ParsedFile;
use crate::syncdoc_debug;
use proc_macro2::TokenStream;
use quote::quote;
use syncdoc_core::parse::*;
use unsynn::*;

pub fn inject_all_doc_comments(
    content: &ModuleContent,
    docs_root: &str,
    parsed: &ParsedFile,
) -> TokenStream {
    syncdoc_debug!("\n=== INJECT_ALL_DOC_COMMENTS START ===");
    syncdoc_debug!("Processing file: {:?}", parsed.path);
    syncdoc_debug!("Number of items: {}", content.items.0.len());

    let mut output = TokenStream::new();

    // ALWAYS inject module doc FIRST if it exists
    if let Some(doc_content) = super::read_module_doc(parsed, docs_root) {
        output.extend(super::generate_module_doc_comments(&doc_content));
    }

    // THEN add non-doc inner attributes (strip out any existing doc attributes)
    if let Some(inner_attrs) = &content.inner_attrs {
        for attr in &inner_attrs.0 {
            // Check if this specific attribute is a module_doc macro or doc attribute
            let mut attr_ts = TokenStream::new();
            unsynn::ToTokens::to_tokens(&attr.value, &mut attr_ts);
            let attr_str = attr_ts.to_string().replace(" ", "");

            // Skip module_doc! macros and doc attributes
            if attr_str.contains("module_doc!")
                || attr_str.contains("#[doc=")
                || attr_str.contains("#![doc=")
            {
                continue;
            }

            // Keep all other inner attributes
            quote::ToTokens::to_tokens(&attr.value, &mut output);
        }
    }

    // Build initial context from the file's module path
    let module_path = syncdoc_core::path_utils::extract_module_path(&parsed.path.to_string_lossy());
    let mut context = Vec::new();
    if !module_path.is_empty() {
        context.push(module_path);
    }

    for item_delimited in &content.items.0 {
        syncdoc_debug!("\n--- Processing item ---");
        syncdoc_debug!(
            "Item type: {:?}",
            std::mem::discriminant(&item_delimited.value)
        );

        output.extend(inject_item_docs(
            &item_delimited.value,
            context.clone(),
            docs_root,
        ));
    }
    syncdoc_debug!("=== INJECT_ALL_DOC_COMMENTS END ===\n");

    output
}

pub(crate) fn inject_item_docs(
    item: &ModuleItem,
    context: Vec<String>,
    docs_root: &str,
) -> TokenStream {
    syncdoc_debug!("\n=== INJECT_ITEM_DOCS ===");
    syncdoc_debug!("Context: {:?}", context);

    match item {
        ModuleItem::TraitMethod(method) => inject_trait_method_docs(method, &context, docs_root),
        ModuleItem::Function(func) => inject_function_docs(func, &context, docs_root),
        ModuleItem::Struct(s) => inject_struct_docs(s, &context, docs_root),
        ModuleItem::Enum(e) => inject_enum_docs(e, &context, docs_root),
        ModuleItem::Module(m) => inject_module_docs(m, context, docs_root),
        ModuleItem::Trait(t) => inject_trait_docs(t, context, docs_root),
        ModuleItem::ImplBlock(i) => inject_impl_docs(i, context, docs_root),
        ModuleItem::TypeAlias(ta) => {
            let mut output = TokenStream::new();

            if let Some(content) =
                super::read_item_markdown(&context, &ta.name.to_string(), docs_root)
            {
                output.extend(super::generate_doc_comments(&content));
            }

            add_non_omnidoc_attrs(&ta.attributes, &mut output);

            if let Some(vis) = &ta.visibility {
                quote::ToTokens::to_tokens(vis, &mut output);
            }
            unsynn::ToTokens::to_tokens(&ta._type, &mut output);
            quote::ToTokens::to_tokens(&ta.name, &mut output);
            if let Some(generics) = &ta.generics {
                unsynn::ToTokens::to_tokens(generics, &mut output);
            }
            unsynn::ToTokens::to_tokens(&ta._eq, &mut output);
            unsynn::ToTokens::to_tokens(&ta.target, &mut output);
            unsynn::ToTokens::to_tokens(&ta._semi, &mut output);

            output
        }
        ModuleItem::Const(c) => {
            syncdoc_debug!("Processing Const: {}", c.name);
            let mut output = TokenStream::new();

            if let Some(content) =
                super::read_item_markdown(&context, &c.name.to_string(), docs_root)
            {
                output.extend(super::generate_doc_comments(&content));
            }

            add_non_omnidoc_attrs(&c.attributes, &mut output);

            if let Some(vis) = &c.visibility {
                quote::ToTokens::to_tokens(vis, &mut output);
            }
            unsynn::ToTokens::to_tokens(&c._const, &mut output);
            quote::ToTokens::to_tokens(&c.name, &mut output);
            unsynn::ToTokens::to_tokens(&c._colon, &mut output);
            unsynn::ToTokens::to_tokens(&c.const_type, &mut output);
            unsynn::ToTokens::to_tokens(&c._eq, &mut output);
            unsynn::ToTokens::to_tokens(&c.value, &mut output);
            unsynn::ToTokens::to_tokens(&c._semi, &mut output);

            output
        }
        ModuleItem::Static(s) => {
            syncdoc_debug!("Processing Static: {}", s.name);
            let mut output = TokenStream::new();

            if let Some(content) =
                super::read_item_markdown(&context, &s.name.to_string(), docs_root)
            {
                output.extend(super::generate_doc_comments(&content));
            }

            add_non_omnidoc_attrs(&s.attributes, &mut output);

            if let Some(vis) = &s.visibility {
                quote::ToTokens::to_tokens(vis, &mut output);
            }
            if let Some(mut_kw) = &s.mut_kw {
                unsynn::ToTokens::to_tokens(mut_kw, &mut output);
            }
            unsynn::ToTokens::to_tokens(&s._static, &mut output);
            quote::ToTokens::to_tokens(&s.name, &mut output);
            unsynn::ToTokens::to_tokens(&s._colon, &mut output);
            unsynn::ToTokens::to_tokens(&s.static_type, &mut output);
            unsynn::ToTokens::to_tokens(&s._eq, &mut output);
            unsynn::ToTokens::to_tokens(&s.value, &mut output);
            unsynn::ToTokens::to_tokens(&s._semi, &mut output);

            output
        }
        ModuleItem::Other(t) => {
            syncdoc_debug!("Processing Other item");
            syncdoc_debug!("Other item tokens: {:?}", t);
            let mut ts = TokenStream::new();
            t.to_tokens(&mut ts);
            ts
        }
    }
}

pub(crate) fn inject_trait_method_docs(
    method: &TraitMethodSig,
    context: &[String],
    docs_root: &str,
) -> TokenStream {
    let mut output = TokenStream::new();

    if let Some(content) = super::read_item_markdown(context, &method.name.to_string(), docs_root) {
        output.extend(super::generate_doc_comments(&content));
    }

    add_non_omnidoc_attrs(&method.attributes, &mut output);

    // Manually reconstruct method signature WITHOUT attributes
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

pub(crate) fn inject_function_docs(
    func: &FnSig,
    context: &[String],
    docs_root: &str,
) -> TokenStream {
    let mut output = TokenStream::new();

    if let Some(content) = super::read_item_markdown(context, &func.name.to_string(), docs_root) {
        output.extend(super::generate_doc_comments(&content));
    }

    add_non_omnidoc_attrs(&func.attributes, &mut output);

    // Manually reconstruct function WITHOUT attributes
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

pub(crate) fn inject_struct_docs(
    struct_sig: &StructSig,
    context: &[String],
    docs_root: &str,
) -> TokenStream {
    let mut output = TokenStream::new();
    let struct_name = struct_sig.name.to_string();

    if let Some(content) = super::read_item_markdown(context, &struct_name, docs_root) {
        output.extend(super::generate_doc_comments(&content));
    }

    add_non_omnidoc_attrs(&struct_sig.attributes, &mut output);

    // Manually reconstruct struct WITHOUT attributes
    if let Some(vis) = &struct_sig.visibility {
        quote::ToTokens::to_tokens(vis, &mut output);
    }
    struct_sig._struct.to_tokens(&mut output);
    quote::ToTokens::to_tokens(&struct_sig.name, &mut output);
    if let Some(generics) = &struct_sig.generics {
        generics.to_tokens(&mut output);
    }
    if let Some(where_clause) = &struct_sig.where_clause {
        where_clause.to_tokens(&mut output);
    }

    match &struct_sig.body {
        StructBody::Named(fields) => {
            let restored_fields = inject_struct_fields(fields, &struct_name, context, docs_root);
            output.extend(wrap_in_braces(restored_fields));
        }
        other => {
            quote::ToTokens::to_tokens(other, &mut output);
        }
    }

    output
}

pub(crate) fn inject_struct_fields(
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

            let mut field_context = context.to_vec();
            field_context.push(struct_name.to_string());
            if let Some(content) = super::read_item_markdown(&field_context, &field_name, docs_root)
            {
                output.extend(super::generate_doc_comments(&content));
            }

            add_non_omnidoc_attrs(&field.attributes, &mut output);

            // Manually reconstruct field WITHOUT attributes
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

pub(crate) fn inject_enum_docs(
    enum_sig: &EnumSig,
    context: &[String],
    docs_root: &str,
) -> TokenStream {
    let mut output = TokenStream::new();
    let enum_name = enum_sig.name.to_string();

    if let Some(content) = super::read_item_markdown(context, &enum_name, docs_root) {
        output.extend(super::generate_doc_comments(&content));
    }

    add_non_omnidoc_attrs(&enum_sig.attributes, &mut output);

    // Manually reconstruct enum WITHOUT attributes
    if let Some(vis) = &enum_sig.visibility {
        quote::ToTokens::to_tokens(vis, &mut output);
    }
    enum_sig._enum.to_tokens(&mut output);
    quote::ToTokens::to_tokens(&enum_sig.name, &mut output);
    if let Some(generics) = &enum_sig.generics {
        generics.to_tokens(&mut output);
    }
    if let Some(where_clause) = &enum_sig.where_clause {
        where_clause.to_tokens(&mut output);
    }

    let restored_variants =
        inject_enum_variants(&enum_sig.variants, &enum_name, context, docs_root);
    output.extend(wrap_in_braces(restored_variants));

    output
}

pub(crate) fn inject_enum_variants(
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

            let mut variant_context = context.to_vec();
            variant_context.push(enum_name.to_string());
            if let Some(content) =
                super::read_item_markdown(&variant_context, &variant_name, docs_root)
            {
                output.extend(super::generate_doc_comments(&content));
            }

            add_non_omnidoc_attrs(&variant.attributes, &mut output);

            // Manually reconstruct variant WITHOUT attributes
            quote::ToTokens::to_tokens(&variant.name, &mut output);

            if let Some(data) = &variant.data {
                match data {
                    EnumVariantData::Struct(fields_containing) => {
                        if let Some(fields_cdv) = fields_containing.content.as_ref() {
                            let restored_fields = inject_enum_variant_fields(
                                fields_cdv,
                                enum_name,
                                &variant_name,
                                context,
                                docs_root,
                            );
                            output.extend(wrap_in_braces(restored_fields));
                        } else {
                            fields_containing.to_tokens(&mut output);
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

pub(crate) fn inject_enum_variant_fields(
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

        let mut field_context = context.to_vec();
        field_context.push(enum_name.to_string());
        field_context.push(variant_name.to_string());
        if let Some(content) = super::read_item_markdown(&field_context, &field_name, docs_root) {
            output.extend(super::generate_doc_comments(&content));
        }

        add_non_omnidoc_attrs(&field.attributes, &mut output);

        // Manually reconstruct field WITHOUT attributes
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

pub(crate) fn inject_module_docs(
    module: &ModuleSig,
    context: Vec<String>,
    docs_root: &str,
) -> TokenStream {
    let mut output = TokenStream::new();
    let module_name = module.name.to_string();

    if let Some(content) = super::read_item_markdown(&context, &module_name, docs_root) {
        output.extend(super::generate_doc_comments(&content));
    }

    add_non_omnidoc_attrs(&module.attributes, &mut output);

    if let Some(vis) = &module.visibility {
        quote::ToTokens::to_tokens(vis, &mut output);
    }
    module._mod.to_tokens(&mut output);
    quote::ToTokens::to_tokens(&module.name, &mut output);

    let mut new_context = context;
    new_context.push(module_name);

    let mut module_content = TokenStream::new();
    for item_delimited in &module.items.content.items.0 {
        module_content.extend(inject_item_docs(
            &item_delimited.value,
            new_context.clone(),
            docs_root,
        ));
    }

    output.extend(wrap_in_braces(module_content));
    output
}

pub(crate) fn inject_trait_docs(
    trait_def: &TraitSig,
    context: Vec<String>,
    docs_root: &str,
) -> TokenStream {
    let mut output = TokenStream::new();
    let trait_name = trait_def.name.to_string();

    syncdoc_debug!("\n=== INJECT TRAIT DEBUG ===");
    syncdoc_debug!("Trait name: {}", trait_name);
    syncdoc_debug!("Context: {:?}", context);
    syncdoc_debug!("Docs root: {}", docs_root);

    let md_content = super::read_item_markdown(&context, &trait_name, docs_root);
    syncdoc_debug!("Found markdown: {}", md_content.is_some());
    if let Some(ref content) = md_content {
        syncdoc_debug!("Content length: {}", content.len());
        syncdoc_debug!("Content: {:?}", &content[..content.len().min(100)]);
    }
    syncdoc_debug!("========================\n");

    if let Some(content) = md_content {
        output.extend(super::generate_doc_comments(&content));
    }

    add_non_omnidoc_attrs(&trait_def.attributes, &mut output);

    if let Some(vis) = &trait_def.visibility {
        quote::ToTokens::to_tokens(vis, &mut output);
    }
    if let Some(unsafe_kw) = &trait_def.unsafe_kw {
        unsafe_kw.to_tokens(&mut output);
    }
    trait_def._trait.to_tokens(&mut output);
    quote::ToTokens::to_tokens(&trait_def.name, &mut output);
    if let Some(generics) = &trait_def.generics {
        generics.to_tokens(&mut output);
    }
    if let Some(bounds) = &trait_def.bounds {
        bounds.to_tokens(&mut output);
    }
    if let Some(where_clause) = &trait_def.where_clause {
        where_clause.to_tokens(&mut output);
    }

    let mut new_context = context;
    new_context.push(trait_name);

    let mut trait_content = TokenStream::new();
    for item_delimited in &trait_def.items.content.items.0 {
        trait_content.extend(inject_item_docs(
            &item_delimited.value,
            new_context.clone(),
            docs_root,
        ));
    }

    output.extend(wrap_in_braces(trait_content));
    output
}

pub(crate) fn inject_impl_docs(
    impl_block: &ImplBlockSig,
    context: Vec<String>,
    docs_root: &str,
) -> TokenStream {
    let mut output = TokenStream::new();

    add_non_omnidoc_attrs(&impl_block.attributes, &mut output);

    impl_block._impl.to_tokens(&mut output);
    if let Some(generics) = &impl_block.generics {
        generics.to_tokens(&mut output);
    }
    impl_block.target_type.to_tokens(&mut output);
    if let Some(for_trait) = &impl_block.for_trait {
        for_trait.to_tokens(&mut output);
    }
    if let Some(where_clause) = &impl_block.where_clause {
        where_clause.to_tokens(&mut output);
    }

    let impl_context = if let Some(for_trait) = &impl_block.for_trait {
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

    let mut impl_content = TokenStream::new();
    for item_delimited in &impl_block.items.content.items.0 {
        impl_content.extend(inject_item_docs(
            &item_delimited.value,
            new_context.clone(),
            docs_root,
        ));
    }

    output.extend(wrap_in_braces(impl_content));
    output
}

pub(crate) fn add_non_omnidoc_attrs(attrs: &Option<Many<Attribute>>, output: &mut TokenStream) {
    if let Some(attr_list) = attrs {
        for attr in &attr_list.0 {
            if !super::is_omnidoc_attr(&attr.value) {
                quote::ToTokens::to_tokens(&attr.value, output);
            }
        }
    }
}

pub(crate) fn wrap_in_braces(content: TokenStream) -> TokenStream {
    let group = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, content);
    std::iter::once(proc_macro2::TokenTree::Group(group)).collect()
}
