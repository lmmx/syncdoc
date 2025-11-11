/// syncdoc-core: documentation injection helper macros
mod config;
mod parse;
mod token_processors;

mod doc_injector;
pub use doc_injector::{syncdoc_impl, inject_doc_attr};

mod omnibus;
pub use omnibus::inject_all_docs_impl;
