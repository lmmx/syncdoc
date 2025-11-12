// syncdoc-migrate/src/lib.rs

mod extract;
mod discover;
mod write;
mod rewrite;
mod report;

pub use discover::{discover_rust_files, parse_file, ParsedFile, get_or_create_docs_path};
pub use extract::{extract_doc_content, has_doc_attrs};
pub use write::{extract_all_docs, write_extractions, DocExtraction, WriteReport};

pub fn migrate() {
    // implementation will go here
}
