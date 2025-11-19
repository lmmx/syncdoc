//! A tree-sitter document section editor.
//!
//! asterism uses ratatui to provide hierarchical navigation of markdown documents
//! and edtui to emulate a vim editor for section content editing.
#![allow(clippy::multiple_crate_versions)]

pub mod app_state;
pub mod config;
pub mod edit_plan;
pub mod formats;
pub mod highlight;
pub mod input;
pub mod section;
pub mod ui;
