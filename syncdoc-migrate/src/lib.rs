// syncdoc-migrate/src/lib.rs

pub mod config;
pub mod debug;
pub mod discover;
mod extract;
mod report;
pub mod restore;
pub mod rewrite;
pub mod write;

pub use config::DocsPathMode;
pub use discover::{discover_rust_files, get_or_create_docs_path, parse_file, ParsedFile};
pub use extract::{extract_doc_content, has_doc_attrs};
pub use restore::restore_file;
pub use rewrite::{inject_module_doc_attr, inject_omnidoc_attr, rewrite_file, strip_doc_attrs};
pub use write::{
    extract_all_docs, find_expected_doc_paths, write_extractions, DocExtraction, WriteReport,
};

/// Macro for debug output in syncdoc.
///
/// Prints to stderr only if debug output is enabled via the atomic flag (tests do this using ctor)
/// or the `SYNCDOC_DEBUG` environment variable at startup.
#[macro_export]
macro_rules! syncdoc_debug {
    ($($arg:tt)*) => {
        if $crate::debug::is_enabled() {
            eprintln!("[SYNCDOC DEBUG] {}", format!($($arg)*));
        }
    };
}

#[cfg(test)]
mod tests;
