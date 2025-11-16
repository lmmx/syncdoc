//! Expected documentation paths based on code structure
//!
//! This module discovers what documentation files should exist based on the
//! structure of Rust source code, regardless of whether documentation comments
//! are present. It's used to identify missing documentation files that should
//! be created.

use crate::write::DocExtraction;
use std::path::{Path, PathBuf};
use syncdoc_core::parse::{
    EnumSig, EnumVariantData, ImplBlockSig, ModuleItem, ModuleSig, StructSig, TraitSig,
};

use super::ParsedFile;

/// Finds all documentation paths that are expected based on code structure
///
/// Returns a vector of `DocExtraction` structs with empty content, representing
/// the markdown files that should exist for the given source file's structure.
pub fn find_expected_doc_paths(parsed: &ParsedFile, docs_root: &str) -> Vec<DocExtraction> {
    let mut extractions = Vec::new();
    let module_path = syncdoc_core::path_utils::extract_module_path(&parsed.path.to_string_lossy());

    // Module-level documentation path
    let file_stem = parsed
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module");

    let path = if module_path.is_empty() {
        format!("{}/{}.md", docs_root, file_stem)
    } else {
        format!("{}/{}.md", docs_root, module_path)
    };

    extractions.push(DocExtraction::new(
        PathBuf::from(path),
        String::new(),
        format!("{}:1", parsed.path.display()),
    ));

    let mut context = Vec::new();
    if !module_path.is_empty() {
        context.push(module_path);
    }

    // Find all item documentation paths
    for item_delimited in &parsed.content.items.0 {
        extractions.extend(find_item_paths(
            &item_delimited.value,
            context.clone(),
            docs_root,
            &parsed.path,
        ));
    }

    extractions
}

/// Recursively finds documentation paths for a single item
fn find_item_paths(
    item: &ModuleItem,
    context: Vec<String>,
    base_path: &str,
    source_file: &Path,
) -> Vec<DocExtraction> {
    let mut extractions = Vec::new();

    match item {
        ModuleItem::TraitMethod(method_sig) => {
            let path = build_path(base_path, &context, &method_sig.name.to_string());
            let location = format!(
                "{}:{}",
                source_file.display(),
                method_sig.name.span().start().line
            );
            extractions.push(DocExtraction::new(
                PathBuf::from(path),
                String::new(),
                location,
            ));
        }

        ModuleItem::Function(func_sig) => {
            let path = build_path(base_path, &context, &func_sig.name.to_string());
            let location = format!(
                "{}:{}",
                source_file.display(),
                func_sig.name.span().start().line
            );
            extractions.push(DocExtraction::new(
                PathBuf::from(path),
                String::new(),
                location,
            ));
        }

        ModuleItem::ImplBlock(impl_block) => {
            extractions.extend(find_impl_paths(impl_block, context, base_path, source_file));
        }

        ModuleItem::Module(module) => {
            extractions.extend(find_module_paths(module, context, base_path, source_file));
        }

        ModuleItem::Trait(trait_def) => {
            extractions.extend(find_trait_paths(trait_def, context, base_path, source_file));
        }

        ModuleItem::Enum(enum_sig) => {
            extractions.extend(find_enum_paths(enum_sig, context, base_path, source_file));
        }

        ModuleItem::Struct(struct_sig) => {
            extractions.extend(find_struct_paths(
                struct_sig,
                context,
                base_path,
                source_file,
            ));
        }

        ModuleItem::TypeAlias(type_alias) => {
            let path = build_path(base_path, &context, &type_alias.name.to_string());
            let location = format!(
                "{}:{}",
                source_file.display(),
                type_alias.name.span().start().line
            );
            extractions.push(DocExtraction::new(
                PathBuf::from(path),
                String::new(),
                location,
            ));
        }

        ModuleItem::Const(const_sig) => {
            let path = build_path(base_path, &context, &const_sig.name.to_string());
            let location = format!(
                "{}:{}",
                source_file.display(),
                const_sig.name.span().start().line
            );
            extractions.push(DocExtraction::new(
                PathBuf::from(path),
                String::new(),
                location,
            ));
        }

        ModuleItem::Static(static_sig) => {
            let path = build_path(base_path, &context, &static_sig.name.to_string());
            let location = format!(
                "{}:{}",
                source_file.display(),
                static_sig.name.span().start().line
            );
            extractions.push(DocExtraction::new(
                PathBuf::from(path),
                String::new(),
                location,
            ));
        }

        ModuleItem::Other(_) => {}
    }

    extractions
}

fn find_impl_paths(
    impl_block: &ImplBlockSig,
    context: Vec<String>,
    base_path: &str,
    source_file: &Path,
) -> Vec<DocExtraction> {
    let mut extractions = Vec::new();

    // Determine the context path for the impl block
    // If this is `impl Trait for Type`, context is [Type, Trait]
    // If this is `impl Type`, context is [Type]
    let impl_context = if let Some(for_trait) = &impl_block.for_trait {
        // This is `impl Trait for Type`
        // target_type contains the TRAIT name (before "for")
        let trait_name = if let Some(first) = impl_block.target_type.0.first() {
            if let proc_macro2::TokenTree::Ident(ident) = &first.value.second {
                ident.to_string()
            } else {
                "Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
        };

        // for_trait.second contains the TYPE name (after "for")
        let type_name = if let Some(first) = for_trait.second.0.first() {
            if let proc_macro2::TokenTree::Ident(ident) = &first.value.second {
                ident.to_string()
            } else {
                "Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
        };

        // Context is Type/Trait
        vec![type_name, trait_name]
    } else {
        // This is `impl Type`, extract Type from target_type
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

    let module_content = &impl_block.items.content;
    for item_delimited in &module_content.items.0 {
        extractions.extend(find_item_paths(
            &item_delimited.value,
            new_context.clone(),
            base_path,
            source_file,
        ));
    }

    extractions
}

fn find_module_paths(
    module: &ModuleSig,
    context: Vec<String>,
    base_path: &str,
    source_file: &Path,
) -> Vec<DocExtraction> {
    let mut extractions = Vec::new();

    let path = build_path(base_path, &context, &module.name.to_string());
    let location = format!(
        "{}:{}",
        source_file.display(),
        module.name.span().start().line
    );
    extractions.push(DocExtraction::new(
        PathBuf::from(path),
        String::new(),
        location,
    ));

    let mut new_context = context;
    new_context.push(module.name.to_string());

    let module_content = &module.items.content;
    for item_delimited in &module_content.items.0 {
        extractions.extend(find_item_paths(
            &item_delimited.value,
            new_context.clone(),
            base_path,
            source_file,
        ));
    }

    extractions
}

fn find_trait_paths(
    trait_def: &TraitSig,
    context: Vec<String>,
    base_path: &str,
    source_file: &Path,
) -> Vec<DocExtraction> {
    let mut extractions = Vec::new();

    let path = build_path(base_path, &context, &trait_def.name.to_string());
    let location = format!(
        "{}:{}",
        source_file.display(),
        trait_def.name.span().start().line
    );
    extractions.push(DocExtraction::new(
        PathBuf::from(path),
        String::new(),
        location,
    ));

    let mut new_context = context;
    new_context.push(trait_def.name.to_string());

    let module_content = &trait_def.items.content;
    for item_delimited in &module_content.items.0 {
        extractions.extend(find_item_paths(
            &item_delimited.value,
            new_context.clone(),
            base_path,
            source_file,
        ));
    }

    extractions
}

fn find_enum_paths(
    enum_sig: &EnumSig,
    context: Vec<String>,
    base_path: &str,
    source_file: &Path,
) -> Vec<DocExtraction> {
    let mut extractions = Vec::new();
    let enum_name = enum_sig.name.to_string();

    let path = build_path(base_path, &context, &enum_name);
    let location = format!(
        "{}:{}",
        source_file.display(),
        enum_sig.name.span().start().line
    );
    extractions.push(DocExtraction::new(
        PathBuf::from(path),
        String::new(),
        location,
    ));

    // Access parsed variants directly
    if let Some(variants_cdv) = enum_sig.variants.content.as_ref() {
        for variant_delimited in &variants_cdv.0 {
            let variant = &variant_delimited.value;
            let path = build_path(
                base_path,
                &context,
                &format!("{}/{}", enum_name, variant.name),
            );
            extractions.push(DocExtraction::new(
                PathBuf::from(path),
                String::new(),
                format!(
                    "{}:{}",
                    source_file.display(),
                    variant.name.span().start().line
                ),
            ));

            // Handle struct variant fields
            if let Some(EnumVariantData::Struct(fields_containing)) = &variant.data {
                if let Some(fields_cdv) = fields_containing.content.as_ref() {
                    for field_delimited in &fields_cdv.0 {
                        let field = &field_delimited.value;
                        let path = build_path(
                            base_path,
                            &context,
                            &format!("{}/{}/{}", enum_name, variant.name, field.name),
                        );
                        extractions.push(DocExtraction::new(
                            PathBuf::from(path),
                            String::new(),
                            format!(
                                "{}:{}",
                                source_file.display(),
                                field.name.span().start().line
                            ),
                        ));
                    }
                }
            }
        }
    }

    extractions
}

fn find_struct_paths(
    struct_sig: &StructSig,
    context: Vec<String>,
    base_path: &str,
    source_file: &Path,
) -> Vec<DocExtraction> {
    let mut extractions = Vec::new();
    let struct_name = struct_sig.name.to_string();

    let path = build_path(base_path, &context, &struct_name);
    let location = format!(
        "{}:{}",
        source_file.display(),
        struct_sig.name.span().start().line
    );
    extractions.push(DocExtraction::new(
        PathBuf::from(path),
        String::new(),
        location,
    ));

    if let syncdoc_core::parse::StructBody::Named(fields_containing) = &struct_sig.body {
        if let Some(fields_cdv) = fields_containing.content.as_ref() {
            for field_delimited in &fields_cdv.0 {
                let field = &field_delimited.value;
                let path = build_path(
                    base_path,
                    &context,
                    &format!("{}/{}", struct_name, field.name),
                );
                extractions.push(DocExtraction::new(
                    PathBuf::from(path),
                    String::new(),
                    format!(
                        "{}:{}",
                        source_file.display(),
                        field.name.span().start().line
                    ),
                ));
            }
        }
    }

    extractions
}

fn build_path(base_path: &str, context: &[String], item_name: &str) -> String {
    let mut parts = vec![base_path.to_string()];
    parts.extend(context.iter().cloned());
    parts.push(format!("{}.md", item_name));
    parts.join("/")
}

#[cfg(test)]
mod expected_tests;
