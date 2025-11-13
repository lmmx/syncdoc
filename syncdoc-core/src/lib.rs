/// syncdoc-core: documentation injection helper macros
pub mod config;
mod doc_injector;
mod omnibus;
pub mod parse;
pub mod path_utils;
mod token_processors;

pub use doc_injector::{inject_doc_attr, syncdoc_impl};
pub use omnibus::inject_all_docs_impl;

#[macro_export]
macro_rules! syncdoc_debug {
    ($($arg:tt)*) => {
        if std::env::var("SYNCDOC_DEBUG").is_ok() {
            eprintln!("[SYNCDOC DEBUG] {}", format!($($arg)*));
        }
    };
}
