// syncdoc-migrate/src/write.rs

use crate::discover::ParsedFile;
use crate::extract::{extract_doc_content, has_doc_attrs};
use proc_macro2::TokenStream;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use syncdoc_core::parse::{EnumSig, ImplBlockSig, ModuleItem, ModuleSig, StructSig, TraitSig};
use unsynn::*;


// Type alias for the complex impl block target type
type ImplTargetType = Many<Cons<Except<Either<syncdoc_core::parse::KFor, BraceGroup>>, proc_macro2::TokenTree>>;

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
    let base_path = docs_root.to_string();

    for item_delimited in &parsed.content.items.0 {
        let item = &item_delimited.value;
        extractions.extend(extract_item_docs(
            item,
            context.clone(),
            &base_path,
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
                let location = format!("{}:{}", source_file.display(), get_span_line(&func_sig.name));
                extractions.push(DocExtraction {
                    markdown_path: PathBuf::from(path),
                    content,
                    source_location: location,
                });
            }
        }

        ModuleItem::ImplBlock(impl_block) => {
            extractions.extend(extract_impl_docs(impl_block, context, base_path, source_file));
        }

        ModuleItem::Module(module) => {
            extractions.extend(extract_module_docs(module, context, base_path, source_file));
        }

        ModuleItem::Trait(trait_def) => {
            extractions.extend(extract_trait_docs(trait_def, context, base_path, source_file));
        }

        ModuleItem::Enum(enum_sig) => {
            extractions.extend(extract_enum_docs(enum_sig, context, base_path, source_file));
        }

        ModuleItem::Struct(struct_sig) => {
            extractions.extend(extract_struct_docs(struct_sig, context, base_path, source_file));
        }

        ModuleItem::TypeAlias(type_alias) => {
            if let Some(content) = extract_doc_content(&type_alias.attributes) {
                let path = build_path(base_path, &context, &type_alias.name.to_string());
                let location = format!("{}:{}", source_file.display(), get_span_line(&type_alias.name));
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
                let location = format!("{}:{}", source_file.display(), get_span_line(&const_sig.name));
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
                let location = format!("{}:{}", source_file.display(), get_span_line(&static_sig.name));
                extractions.push(DocExtraction {
                    markdown_path: PathBuf::from(path),
                    content,
                    source_location: location,
                });
            }
        }

        ModuleItem::Other(_) => {
            // No documentation to extract from other items
        }
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
    let type_name = extract_type_name(&impl_block.target_type);

    // Update context with the impl type name
    let mut new_context = context;
    new_context.push(type_name);

    // Extract the impl body content
    let body_stream = extract_group_content(&impl_block.body);

    // Parse body as module content and extract from items
    if let Ok(content) = body_stream.into_token_iter().parse::<syncdoc_core::parse::ModuleContent>() {
        for item_delimited in &content.items.0 {
            let item = &item_delimited.value;
            extractions.extend(extract_item_docs(
                item,
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
        let location = format!("{}:{}", source_file.display(), get_span_line(&module.name));
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
    let body_stream = extract_group_content(&module.body);

    // Parse body as module content and extract from items
    if let Ok(content) = body_stream.into_token_iter().parse::<syncdoc_core::parse::ModuleContent>() {
        for item_delimited in &content.items.0 {
            let item = &item_delimited.value;
            extractions.extend(extract_item_docs(
                item,
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
        let location = format!("{}:{}", source_file.display(), get_span_line(&trait_def.name));
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
    let body_stream = extract_group_content(&trait_def.body);

    // Parse body and extract from methods (only default methods have docs that matter)
    if let Ok(content) = body_stream.into_token_iter().parse::<syncdoc_core::parse::ModuleContent>() {
        for item_delimited in &content.items.0 {
            let item = &item_delimited.value;
            extractions.extend(extract_item_docs(
                item,
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
        let location = format!("{}:{}", source_file.display(), get_span_line(&enum_sig.name));
        extractions.push(DocExtraction {
            markdown_path: PathBuf::from(path),
            content,
            source_location: location,
        });
    }

    // Extract variant documentation
    let body_stream = extract_group_content(&enum_sig.body);
    let variant_context = vec![enum_name.clone()];

    for variant in parse_enum_variants(body_stream) {
        if let Some(content) = extract_doc_content(&variant.attrs) {
            let path = build_path(base_path, &context, &format!("{}/{}", enum_name, variant.name));
            extractions.push(DocExtraction {
                markdown_path: PathBuf::from(path),
                content,
                source_location: format!("{}:{}", source_file.display(), variant.line),
            });
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
        let location = format!("{}:{}", source_file.display(), get_span_line(&struct_sig.name));
        extractions.push(DocExtraction {
            markdown_path: PathBuf::from(path),
            content,
            source_location: location,
        });
    }

    // Extract field documentation (only for named fields)
    if let syncdoc_core::parse::StructBody::Named(brace_group) = &struct_sig.body {
        let body_stream = extract_group_content(brace_group);

        for field in parse_struct_fields(body_stream) {
            if let Some(content) = extract_doc_content(&field.attrs) {
                let path = build_path(base_path, &context, &format!("{}/{}", struct_name, field.name));
                extractions.push(DocExtraction {
                    markdown_path: PathBuf::from(path),
                    content,
                    source_location: format!("{}:{}", source_file.display(), field.line),
                });
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

fn extract_type_name(target_type: &ImplTargetType) -> String {
    if let Some(first) = target_type.0.first() {
        if let proc_macro2::TokenTree::Ident(ident) = &first.value.second {
            return ident.to_string();
        }
    }
    "Unknown".to_string()
}

fn extract_group_content(group: &impl quote::ToTokens) -> TokenStream {
    let mut ts = TokenStream::new();
    group.to_tokens(&mut ts);
    if let Some(proc_macro2::TokenTree::Group(g)) = ts.into_iter().next() {
        g.stream()
    } else {
        TokenStream::new()
    }
}

fn get_span_line(ident: &proc_macro2::Ident) -> usize {
    ident.span().start().line
}

// Simplified variant/field parsing structures

#[derive(Debug)]
struct VariantInfo {
    name: String,
    attrs: Option<unsynn::Many<syncdoc_core::parse::Attribute>>,
    line: usize,
}

#[derive(Debug)]
struct FieldInfo {
    name: String,
    attrs: Option<unsynn::Many<syncdoc_core::parse::Attribute>>,
    line: usize,
}

fn parse_enum_variants(body_stream: TokenStream) -> Vec<VariantInfo> {
    let mut variants = Vec::new();
    let mut current_attrs: Option<unsynn::Many<syncdoc_core::parse::Attribute>> = None;
    let mut tokens = body_stream.into_iter().peekable();

    while let Some(tt) = tokens.next() {
        match tt {
            proc_macro2::TokenTree::Punct(ref punct) if punct.as_char() == '#' => {
                // Start of attribute
                if let Some(proc_macro2::TokenTree::Group(_)) = tokens.peek() {
                    // Parse attributes - simplified approach
                    // In production, would parse properly using unsynn
                    current_attrs = None; // Placeholder
                }
            }
            proc_macro2::TokenTree::Ident(ident) => {
                // This is a variant name
                let variant = VariantInfo {
                    name: ident.to_string(),
                    attrs: current_attrs.take(),
                    line: ident.span().start().line,
                };
                variants.push(variant);

                // Skip until comma or end
                while let Some(tt) = tokens.next() {
                    if let proc_macro2::TokenTree::Punct(ref punct) = tt {
                        if punct.as_char() == ',' {
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    variants
}

fn parse_struct_fields(body_stream: TokenStream) -> Vec<FieldInfo> {
    let mut fields = Vec::new();
    let mut current_attrs: Option<unsynn::Many<syncdoc_core::parse::Attribute>> = None;
    let mut tokens = body_stream.into_iter().peekable();

    while let Some(tt) = tokens.next() {
        match tt {
            proc_macro2::TokenTree::Punct(ref punct) if punct.as_char() == '#' => {
                // Start of attribute - simplified
                if let Some(proc_macro2::TokenTree::Group(_)) = tokens.peek() {
                    current_attrs = None; // Placeholder
                }
            }
            proc_macro2::TokenTree::Ident(ident) => {
                let ident_str = ident.to_string();
                // Skip visibility keywords
                if ident_str == "pub" {
                    if let Some(proc_macro2::TokenTree::Group(_)) = tokens.peek() {
                        tokens.next(); // Skip (crate) or similar
                    }
                    continue;
                }

                // This should be a field name
                let field = FieldInfo {
                    name: ident.to_string(),
                    attrs: current_attrs.take(),
                    line: ident.span().start().line,
                };
                fields.push(field);

                // Skip until comma or end
                while let Some(tt) = tokens.next() {
                    if let proc_macro2::TokenTree::Punct(ref punct) = tt {
                        if punct.as_char() == ',' {
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fields
}

#[cfg(test)]
mod tests;
