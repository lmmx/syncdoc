// syncdoc-migrate/src/lib.rs

pub mod discover;
mod extract;
mod report;
pub mod rewrite;
pub mod write;

pub use discover::{discover_rust_files, get_or_create_docs_path, parse_file, ParsedFile};
pub use extract::{extract_doc_content, has_doc_attrs};
pub use rewrite::{inject_omnidoc_attr, rewrite_file, strip_doc_attrs};
pub use write::{extract_all_docs, write_extractions, DocExtraction, WriteReport};
