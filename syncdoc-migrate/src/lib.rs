// syncdoc-migrate/src/lib.rs

pub mod discover;
mod extract;
mod report;
pub mod rewrite;
pub mod write;

pub use discover::{discover_rust_files, get_or_create_docs_path, parse_file, ParsedFile};
pub use extract::{extract_doc_content, has_doc_attrs};
pub use rewrite::{inject_module_doc_attr, inject_omnidoc_attr, rewrite_file, strip_doc_attrs};
pub use write::{
    extract_all_docs, find_expected_doc_paths, write_extractions, DocExtraction, WriteReport,
};

#[macro_export]
macro_rules! syncdoc_debug {
    ($($arg:tt)*) => {
        if std::env::var("SYNCDOC_DEBUG").is_ok() {
            eprintln!("[SYNCDOC DEBUG] {}", format!($($arg)*));
        }
    };
}
