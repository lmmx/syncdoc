//! Documentation injection logic for restore operation
//!
//! Injects inline doc comments by reading from markdown files, removing omnidoc attributes.

use crate::discover::ParsedFile;
use proc_macro2::TokenStream;
use quote::quote;
use syncdoc_core::parse::*;
use unsynn::*;

pub fn inject_all_doc_comments(
    content: &ModuleContent,
    docs_root: &str,
    parsed: &ParsedFile,
) -> TokenStream {
    let mut output = TokenStream::new();

    if let Some(inner_attrs) = &content.inner_attrs {
        if super::is_module_doc_macro(inner_attrs) {
            if let Some(doc_content) = super::read_module_doc(parsed, docs_root) {
                output.extend(super::generate_module_doc_comments(&doc_content));
            }
        } else {
            for attr in &inner_attrs.0 {
                quote::ToTokens::to_tokens(&attr.value, &mut output);
            }
        }
    }

    for item_delimited in &content.items.0 {
        output.extend(inject_item_docs(
            &item_delimited.value,
            Vec::new(),
            docs_root,
        ));
    }

    output
}

fn inject_item_docs(item: &ModuleItem, context: Vec<String>, docs_root: &str) -> TokenStream {
    match item {
        ModuleItem::TraitMethod(method) => inject_trait_method_docs(method, &context, docs_root),
        ModuleItem::Function(func) => inject_function_docs(func, &context, docs_root),
        ModuleItem::Struct(s) => inject_struct_docs(s, &context, docs_root),
        ModuleItem::Enum(e) => inject_enum_docs(e, &context, docs_root),
        ModuleItem::Module(m) => inject_module_docs(m, context, docs_root),
        ModuleItem::Trait(t) => inject_trait_docs(t, context, docs_root),
        ModuleItem::ImplBlock(i) => inject_impl_docs(i, context, docs_root),
        ModuleItem::TypeAlias(ta) => {
            inject_simple_item_docs(ta, &ta.name.to_string(), &context, docs_root)
        }
        ModuleItem::Const(c) => {
            inject_simple_item_docs(c, &c.name.to_string(), &context, docs_root)
        }
        ModuleItem::Static(s) => {
            inject_simple_item_docs(s, &s.name.to_string(), &context, docs_root)
        }
        ModuleItem::Other(t) => {
            let mut ts = TokenStream::new();
            t.to_tokens(&mut ts);
            ts
        }
    }
}

fn inject_trait_method_docs(
    method: &TraitMethodSig,
    context: &[String],
    docs_root: &str,
) -> TokenStream {
    let mut output = TokenStream::new();

    if let Some(content) = super::read_item_markdown(context, &method.name.to_string(), docs_root) {
        output.extend(super::generate_doc_comments(&content));
    }

    add_non_omnidoc_attrs(&method.attributes, &mut output);
    quote::ToTokens::to_tokens(method, &mut output);

    output
}

fn inject_function_docs(func: &FnSig, context: &[String], docs_root: &str) -> TokenStream {
    let mut output = TokenStream::new();

    if let Some(content) = super::read_item_markdown(context, &func.name.to_string(), docs_root) {
        output.extend(super::generate_doc_comments(&content));
    }

    add_non_omnidoc_attrs(&func.attributes, &mut output);
    quote::ToTokens::to_tokens(func, &mut output);

    output
}

fn inject_simple_item_docs<T: quote::ToTokens>(
    item: &T,
    name: &str,
    context: &[String],
    docs_root: &str,
) -> TokenStream {
    let mut output = TokenStream::new();

    if let Some(content) = super::read_item_markdown(context, name, docs_root) {
        output.extend(super::generate_doc_comments(&content));
    }

    quote::ToTokens::to_tokens(item, &mut output);

    output
}

fn inject_struct_docs(struct_sig: &StructSig, context: &[String], docs_root: &str) -> TokenStream {
    let mut output = TokenStream::new();
    let struct_name = struct_sig.name.to_string();

    if let Some(content) = super::read_item_markdown(context, &struct_name, docs_root) {
        output.extend(super::generate_doc_comments(&content));
    }

    add_non_omnidoc_attrs(&struct_sig.attributes, &mut output);

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

fn inject_struct_fields(
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
            quote::ToTokens::to_tokens(field, &mut output);

            if idx < fields_cdv.0.len() - 1 {
                output.extend(quote! { , });
            }
        }
    }

    output
}

fn inject_enum_docs(enum_sig: &EnumSig, context: &[String], docs_root: &str) -> TokenStream {
    let mut output = TokenStream::new();
    let enum_name = enum_sig.name.to_string();

    if let Some(content) = super::read_item_markdown(context, &enum_name, docs_root) {
        output.extend(super::generate_doc_comments(&content));
    }

    add_non_omnidoc_attrs(&enum_sig.attributes, &mut output);

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

fn inject_enum_variants(
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

fn inject_enum_variant_fields(
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
        quote::ToTokens::to_tokens(field, &mut output);

        if idx < fields.0.len() - 1 {
            output.extend(quote! { , });
        }
    }

    output
}

fn inject_module_docs(module: &ModuleSig, context: Vec<String>, docs_root: &str) -> TokenStream {
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

fn inject_trait_docs(trait_def: &TraitSig, context: Vec<String>, docs_root: &str) -> TokenStream {
    let mut output = TokenStream::new();
    let trait_name = trait_def.name.to_string();

    if let Some(content) = super::read_item_markdown(&context, &trait_name, docs_root) {
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

fn inject_impl_docs(
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

fn add_non_omnidoc_attrs(attrs: &Option<Many<Attribute>>, output: &mut TokenStream) {
    if let Some(attr_list) = attrs {
        for attr in &attr_list.0 {
            if !super::is_omnidoc_attr(&attr.value) {
                quote::ToTokens::to_tokens(&attr.value, output);
            }
        }
    }
}

fn wrap_in_braces(content: TokenStream) -> TokenStream {
    let group = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, content);
    std::iter::once(proc_macro2::TokenTree::Group(group)).collect()
}
