// syncdoc-migrate/src/write.rs

use crate::discover::ParsedFile;
use crate::extract::extract_doc_content;
use proc_macro2::TokenStream;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use syncdoc_core::parse::{
    EnumSig, ImplBlockSig, ModuleContent, ModuleItem, ModuleSig, StructSig, TraitSig,
};
use unsynn::*;

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
    let context = Vec::new();

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
                extractions.push(DocExtraction {
                    markdown_path: PathBuf::from(path),
                    content,
                    source_location: location,
                });
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
                extractions.push(DocExtraction {
                    markdown_path: PathBuf::from(path),
                    content,
                    source_location: location,
                });
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
                extractions.push(DocExtraction {
                    markdown_path: PathBuf::from(path),
                    content,
                    source_location: location,
                });
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
                extractions.push(DocExtraction {
                    markdown_path: PathBuf::from(path),
                    content,
                    source_location: location,
                });
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

    // Extract type name from target_type - reuse logic from token_processors.rs
    let type_name = if let Some(first) = impl_block.target_type.0.first() {
        if let proc_macro2::TokenTree::Ident(ident) = &first.value.second {
            ident.to_string()
        } else {
            "Unknown".to_string()
        }
    } else {
        "Unknown".to_string()
    };

    let mut new_context = context;
    new_context.push(type_name);

    // Parse the body content
    let body_stream = extract_brace_content(&impl_block.body);
    if let Ok(content) = body_stream.into_token_iter().parse::<ModuleContent>() {
        for item_delimited in &content.items.0 {
            extractions.extend(extract_item_docs(
                &item_delimited.value,
                new_context.clone(),
                base_path,
                source_file,
            ));
        }
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
        extractions.push(DocExtraction {
            markdown_path: PathBuf::from(path),
            content,
            source_location: location,
        });
    }

    // Update context with module name
    let mut new_context = context;
    new_context.push(module.name.to_string());

    // Extract the module body content
    let body_stream = extract_brace_content(&module.body);
    if let Ok(content) = body_stream.into_token_iter().parse::<ModuleContent>() {
        for item_delimited in &content.items.0 {
            extractions.extend(extract_item_docs(
                &item_delimited.value,
                new_context.clone(),
                base_path,
                source_file,
            ));
        }
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
        extractions.push(DocExtraction {
            markdown_path: PathBuf::from(path),
            content,
            source_location: location,
        });
    }

    // Update context with trait name
    let mut new_context = context;
    new_context.push(trait_def.name.to_string());

    // Extract the trait body content
    let body_stream = extract_brace_content(&trait_def.body);
    if let Ok(content) = body_stream.into_token_iter().parse::<ModuleContent>() {
        for item_delimited in &content.items.0 {
            extractions.extend(extract_item_docs(
                &item_delimited.value,
                new_context.clone(),
                base_path,
                source_file,
            ));
        }
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
        extractions.push(DocExtraction {
            markdown_path: PathBuf::from(path),
            content,
            source_location: location,
        });
    }

    // Extract variant documentation
    let body_stream = extract_brace_content(&enum_sig.body);

    // Parse variants manually since we need their attributes
    let mut tokens = body_stream.into_iter().peekable();
    let mut current_attrs = None;

    while let Some(tt) = tokens.next() {
        match tt {
            proc_macro2::TokenTree::Punct(ref p) if p.as_char() == '#' => {
                // Collect attributes - simplified for now
                if let Some(proc_macro2::TokenTree::Group(_)) = tokens.peek() {
                    tokens.next();
                }
            }
            proc_macro2::TokenTree::Ident(ident) => {
                // This is a variant name
                let variant_name = ident.to_string();
                let path = build_path(
                    base_path,
                    &context,
                    &format!("{}/{}", enum_name, variant_name),
                );

                // Would need to properly parse attributes here
                // For now, skip variant docs - this is a limitation

                // Skip to next comma
                while let Some(tt) = tokens.next() {
                    if let proc_macro2::TokenTree::Punct(ref p) = tt {
                        if p.as_char() == ',' {
                            break;
                        }
                    }
                }
            }
            _ => {}
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
        extractions.push(DocExtraction {
            markdown_path: PathBuf::from(path),
            content,
            source_location: location,
        });
    }

    // Extract field documentation (only for named fields)
    if let syncdoc_core::parse::StructBody::Named(brace_group) = &struct_sig.body {
        // Similar manual parsing needed for fields
        // This is a limitation - would need proper field parsing
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
                .or_insert_with(Vec::new)
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

fn extract_brace_content(brace_group: &BraceGroup) -> TokenStream {
    let mut ts = TokenStream::new();
    unsynn::ToTokens::to_tokens(brace_group, &mut ts);
    if let Some(proc_macro2::TokenTree::Group(g)) = ts.into_iter().next() {
        g.stream()
    } else {
        TokenStream::new()
    }
}

#[cfg(test)]
mod tests;
