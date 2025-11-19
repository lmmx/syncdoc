// syncdoc-migrate/src/lib.rs

pub mod config;
pub mod discover;
mod extract;
mod report;
pub mod restore;
pub mod rewrite;
pub mod write;

// Re-export core's macro
pub use syncdoc_core::syncdoc_debug;

pub use config::DocsPathMode;
pub use discover::{discover_rust_files, get_or_create_docs_path, parse_file, ParsedFile};
pub use extract::{extract_doc_content, has_doc_attrs};
pub use restore::restore_file;
pub use rewrite::{inject_module_doc_attr, inject_omnidoc_attr, rewrite_file, strip_doc_attrs};
pub use write::{
    extract_all_docs, find_expected_doc_paths, write_extractions, DocExtraction, WriteReport,
};

#[cfg(test)]
mod tests;
