/// syncdoc-core: documentation injection helper macros
mod config;
pub mod parse;
mod token_processors;

mod doc_injector;
pub use doc_injector::{inject_doc_attr, syncdoc_impl};

mod omnibus;
pub use omnibus::inject_all_docs_impl;
