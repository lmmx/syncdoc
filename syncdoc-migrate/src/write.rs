//! List all the files we expect to be produced from code with omnidoc attributes.

use crate::discover::ParsedFile;
use crate::extract::extract_doc_content;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use syncdoc_core::parse::{
    EnumSig, EnumVariantData, ImplBlockSig, ModuleItem, ModuleSig, StructSig, TraitSig,
};

mod expected;
pub use expected::find_expected_doc_paths;

/// Represents a documentation extraction with its target path and metadata
#[derive(Debug, Clone, PartialEq)]
pub struct DocExtraction {
    /// Path where the markdown file should be written
    pub markdown_path: PathBuf,
    /// The documentation content to write
    pub content: String,
    /// Source location in file:line format
    pub source_location: String,
}

impl DocExtraction {
    /// Creates a new DocExtraction and ensures content ends with a newline
    pub fn new(markdown_path: PathBuf, mut content: String, source_location: String) -> Self {
        if !content.ends_with('\n') {
            content.push('\n');
        }
        Self {
            markdown_path,
            content,
            source_location,
        }
    }
}

/// Report of write operation results
#[derive(Debug, Default)]
pub struct WriteReport {
    pub files_written: usize,
    pub files_skipped: usize,
    pub errors: Vec<String>,
}

/// Extracts all documentation from a parsed file
///
/// Returns a vector of `DocExtraction` structs, each representing a documentation
/// comment that should be written to a markdown file.
pub fn extract_all_docs(parsed: &ParsedFile, docs_root: &str) -> Vec<DocExtraction> {
    let mut extractions = Vec::new();

    // Extract module path from the source file
    let module_path = syncdoc_core::path_utils::extract_module_path(&parsed.path.to_string_lossy());

    // Extract module-level (inner) documentation
    if let Some(inner_doc) = crate::extract::extract_inner_doc_content(&parsed.content.inner_attrs)
    {
        // For lib.rs -> docs/lib.md, for main.rs -> docs/main.md, etc.
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
            inner_doc,
            format!("{}:1", parsed.path.display()),
        ));
    }

    // Start context with module path if not empty
    let mut context = Vec::new();
    if !module_path.is_empty() {
        context.push(module_path);
    }

    for item_delimited in &parsed.content.items.0 {
        let item = &item_delimited.value;
        extractions.extend(extract_item_docs(
            item,
            context.clone(),
            docs_root,
            &parsed.path,
        ));
    }

    extractions
}

/// Recursively extracts documentation from a single module item
fn extract_item_docs(
    item: &ModuleItem,
    context: Vec<String>,
    base_path: &str,
    source_file: &Path,
) -> Vec<DocExtraction> {
    let mut extractions = Vec::new();

    match item {
        ModuleItem::Function(func_sig) => {
            if let Some(content) = extract_doc_content(&func_sig.attributes) {
                let path = build_path(base_path, &context, &func_sig.name.to_string());
                let location = format!(
                    "{}:{}",
                    source_file.display(),
                    func_sig.name.span().start().line
                );
                extractions.push(DocExtraction::new(PathBuf::from(path), content, location));
            }
        }

        ModuleItem::ImplBlock(impl_block) => {
            extractions.extend(extract_impl_docs(
                impl_block,
                context,
                base_path,
                source_file,
            ));
        }

        ModuleItem::Module(module) => {
            extractions.extend(extract_module_docs(module, context, base_path, source_file));
        }

        ModuleItem::Trait(trait_def) => {
            extractions.extend(extract_trait_docs(
                trait_def,
                context,
                base_path,
                source_file,
            ));
        }

        ModuleItem::Enum(enum_sig) => {
            extractions.extend(extract_enum_docs(enum_sig, context, base_path, source_file));
        }

        ModuleItem::Struct(struct_sig) => {
            extractions.extend(extract_struct_docs(
                struct_sig,
                context,
                base_path,
                source_file,
            ));
        }

        ModuleItem::TypeAlias(type_alias) => {
            if let Some(content) = extract_doc_content(&type_alias.attributes) {
                let path = build_path(base_path, &context, &type_alias.name.to_string());
                let location = format!(
                    "{}:{}",
                    source_file.display(),
                    type_alias.name.span().start().line
                );
                extractions.push(DocExtraction::new(PathBuf::from(path), content, location));
            }
        }

        ModuleItem::Const(const_sig) => {
            if let Some(content) = extract_doc_content(&const_sig.attributes) {
                let path = build_path(base_path, &context, &const_sig.name.to_string());
                let location = format!(
                    "{}:{}",
                    source_file.display(),
                    const_sig.name.span().start().line
                );
                extractions.push(DocExtraction::new(PathBuf::from(path), content, location));
            }
        }

        ModuleItem::Static(static_sig) => {
            if let Some(content) = extract_doc_content(&static_sig.attributes) {
                let path = build_path(base_path, &context, &static_sig.name.to_string());
                let location = format!(
                    "{}:{}",
                    source_file.display(),
                    static_sig.name.span().start().line
                );
                extractions.push(DocExtraction::new(PathBuf::from(path), content, location));
            }
        }

        // No documentation to extract from other items
        ModuleItem::Other(_) => {}
    }

    extractions
}

/// Extracts documentation from an impl block and its methods
fn extract_impl_docs(
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

    // Access parsed items directly
    let module_content = &impl_block.items.content;
    for item_delimited in &module_content.items.0 {
        extractions.extend(extract_item_docs(
            &item_delimited.value,
            new_context.clone(),
            base_path,
            source_file,
        ));
    }

    extractions
}

/// Extracts documentation from a module and its contents
fn extract_module_docs(
    module: &ModuleSig,
    context: Vec<String>,
    base_path: &str,
    source_file: &Path,
) -> Vec<DocExtraction> {
    let mut extractions = Vec::new();

    // Extract module's own documentation if present
    if let Some(content) = extract_doc_content(&module.attributes) {
        let path = build_path(base_path, &context, &module.name.to_string());
        let location = format!(
            "{}:{}",
            source_file.display(),
            module.name.span().start().line
        );
        extractions.push(DocExtraction::new(PathBuf::from(path), content, location));
    }

    // Update context with module name
    let mut new_context = context;
    new_context.push(module.name.to_string());

    // Access parsed items directly
    let module_content = &module.items.content;
    for item_delimited in &module_content.items.0 {
        extractions.extend(extract_item_docs(
            &item_delimited.value,
            new_context.clone(),
            base_path,
            source_file,
        ));
    }

    extractions
}

/// Extracts documentation from a trait and its methods
fn extract_trait_docs(
    trait_def: &TraitSig,
    context: Vec<String>,
    base_path: &str,
    source_file: &Path,
) -> Vec<DocExtraction> {
    let mut extractions = Vec::new();

    // Extract trait's own documentation if present
    if let Some(content) = extract_doc_content(&trait_def.attributes) {
        let path = build_path(base_path, &context, &trait_def.name.to_string());
        let location = format!(
            "{}:{}",
            source_file.display(),
            trait_def.name.span().start().line
        );
        extractions.push(DocExtraction::new(PathBuf::from(path), content, location));
    }

    // Update context with trait name
    let mut new_context = context;
    new_context.push(trait_def.name.to_string());

    // Access parsed items directly
    let module_content = &trait_def.items.content;
    for item_delimited in &module_content.items.0 {
        extractions.extend(extract_item_docs(
            &item_delimited.value,
            new_context.clone(),
            base_path,
            source_file,
        ));
    }

    extractions
}

/// Extracts documentation from an enum and its variants
fn extract_enum_docs(
    enum_sig: &EnumSig,
    context: Vec<String>,
    base_path: &str,
    source_file: &Path,
) -> Vec<DocExtraction> {
    let mut extractions = Vec::new();
    let enum_name = enum_sig.name.to_string();

    // Extract enum's own documentation
    if let Some(content) = extract_doc_content(&enum_sig.attributes) {
        let path = build_path(base_path, &context, &enum_name);
        let location = format!(
            "{}:{}",
            source_file.display(),
            enum_sig.name.span().start().line
        );
        extractions.push(DocExtraction::new(PathBuf::from(path), content, location));
    }

    // Access parsed variants directly
    if let Some(variants_cdv) = enum_sig.variants.content.as_ref() {
        for variant_delimited in &variants_cdv.0 {
            let variant = &variant_delimited.value;
            if let Some(content) = extract_doc_content(&variant.attributes) {
                let path = build_path(
                    base_path,
                    &context,
                    &format!("{}/{}", enum_name, variant.name),
                );
                extractions.push(DocExtraction::new(
                    PathBuf::from(path),
                    content,
                    format!(
                        "{}:{}",
                        source_file.display(),
                        variant.name.span().start().line
                    ),
                ));
            }

            // Handle struct variant fields (Issue #34!)
            if let Some(EnumVariantData::Struct(fields_containing)) = &variant.data {
                if let Some(fields_cdv) = fields_containing.content.as_ref() {
                    for field_delimited in &fields_cdv.0 {
                        let field = &field_delimited.value;
                        if let Some(content) = extract_doc_content(&field.attributes) {
                            let path = build_path(
                                base_path,
                                &context,
                                &format!("{}/{}/{}", enum_name, variant.name, field.name),
                            );
                            extractions.push(DocExtraction::new(
                                PathBuf::from(path),
                                content,
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
    }

    extractions
}

/// Extracts documentation from a struct and its fields
fn extract_struct_docs(
    struct_sig: &StructSig,
    context: Vec<String>,
    base_path: &str,
    source_file: &Path,
) -> Vec<DocExtraction> {
    let mut extractions = Vec::new();
    let struct_name = struct_sig.name.to_string();

    // Extract struct's own documentation
    if let Some(content) = extract_doc_content(&struct_sig.attributes) {
        let path = build_path(base_path, &context, &struct_name);
        let location = format!(
            "{}:{}",
            source_file.display(),
            struct_sig.name.span().start().line
        );
        extractions.push(DocExtraction::new(PathBuf::from(path), content, location));
    }

    // Extract field documentation (only for named fields)
    if let syncdoc_core::parse::StructBody::Named(fields_containing) = &struct_sig.body {
        if let Some(fields_cdv) = fields_containing.content.as_ref() {
            for field_delimited in &fields_cdv.0 {
                let field = &field_delimited.value;
                if let Some(content) = extract_doc_content(&field.attributes) {
                    let path = build_path(
                        base_path,
                        &context,
                        &format!("{}/{}", struct_name, field.name),
                    );
                    extractions.push(DocExtraction::new(
                        PathBuf::from(path),
                        content,
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

    extractions
}

/// Writes documentation extractions to markdown files
///
/// If `dry_run` is true, validates paths and reports what would be written
/// without actually creating files.
pub fn write_extractions(
    extractions: &[DocExtraction],
    dry_run: bool,
) -> std::io::Result<WriteReport> {
    let mut report = WriteReport::default();

    // Group by parent directory for efficient directory creation
    let mut dirs: HashMap<PathBuf, Vec<&DocExtraction>> = HashMap::new();
    for extraction in extractions {
        if let Some(parent) = extraction.markdown_path.parent() {
            dirs.entry(parent.to_path_buf())
                .or_default()
                .push(extraction);
        }
    }

    // Create directories
    for dir in dirs.keys() {
        if !dry_run {
            if let Err(e) = fs::create_dir_all(dir) {
                report.errors.push(format!(
                    "Failed to create directory {}: {}",
                    dir.display(),
                    e
                ));
                continue;
            }
        }
    }

    // Write files
    for extraction in extractions {
        if dry_run {
            println!("Would write: {}", extraction.markdown_path.display());
            report.files_written += 1;
        } else {
            match fs::write(&extraction.markdown_path, &extraction.content) {
                Ok(_) => {
                    report.files_written += 1;
                }
                Err(e) => {
                    report.errors.push(format!(
                        "Failed to write {}: {}",
                        extraction.markdown_path.display(),
                        e
                    ));
                    report.files_skipped += 1;
                }
            }
        }
    }

    Ok(report)
}

// Helper functions

fn build_path(base_path: &str, context: &[String], item_name: &str) -> String {
    let mut parts = vec![base_path.to_string()];
    parts.extend(context.iter().cloned());
    parts.push(format!("{}.md", item_name));
    parts.join("/")
}

#[cfg(test)]
mod tests;
