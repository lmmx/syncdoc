/// syncdoc-core: documentation injection helper macros
pub mod config;
pub mod debug;
mod doc_injector;
mod omnibus;
pub mod parse;
pub mod path_utils;
pub mod token_processors;

pub use doc_injector::{module_doc_impl, omnidoc_impl};
pub use omnibus::inject_all_docs_impl;

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
