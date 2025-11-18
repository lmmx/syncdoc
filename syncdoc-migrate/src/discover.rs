// syncdoc-migrate/src/discover.rs

use crate::config::DocsPathMode;
use proc_macro2::TokenStream;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use syncdoc_core::parse::ModuleContent;
use unsynn::*;

/// Represents a parsed Rust file with its content and metadata
#[derive(Debug)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub content: ModuleContent,
    pub original_source: String,
}

/// Errors that can occur during file parsing
#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    ParseFailed(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(e) => write!(f, "IO error: {}", e),
            ParseError::ParseFailed(msg) => write!(f, "Parse failed: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

/// Errors that can occur during configuration
#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    Other(String),
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Recursively discovers all Rust source files in a directory
///
/// Returns a sorted vector of absolute paths to `.rs` files for deterministic processing.
pub fn discover_rust_files(source_dir: &Path) -> std::result::Result<Vec<PathBuf>, std::io::Error> {
    let mut rust_files = Vec::new();
    discover_rust_files_recursive(source_dir, &mut rust_files)?;
    rust_files.sort();
    Ok(rust_files)
}

fn discover_rust_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> std::result::Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            discover_rust_files_recursive(&path, files)?;
        } else if path.extension() == Some(std::ffi::OsStr::new("rs")) {
            files.push(path.canonicalize()?);
        }
    }
    Ok(())
}

/// Parses a Rust source file into a structured representation
///
/// Returns `ParseError::ParseFailed` if the file cannot be parsed, allowing
/// the caller to skip unparseable files.
pub fn parse_file(path: &Path) -> std::result::Result<ParsedFile, ParseError> {
    let original_source = fs::read_to_string(path)?;

    let token_stream = TokenStream::from_str(&original_source)
        .map_err(|e| ParseError::ParseFailed(format!("Failed to tokenize: {}", e)))?;

    let content = token_stream
        .into_token_iter()
        .parse::<ModuleContent>()
        .map_err(|e| ParseError::ParseFailed(format!("Failed to parse module: {}", e)))?;

    Ok(ParsedFile {
        path: path.to_path_buf(),
        content,
        original_source,
    })
}

/// Gets or creates the docs-path configuration
///
/// Returns a tuple of (path, mode) where mode indicates whether the path
/// is configured via TOML or should be inlined.
///
/// If the docs-path is not set in Cargo.toml, this function will append
/// the default configuration and return "docs" (unless dry_run is true).
pub fn get_or_create_docs_path(
    source_file: &Path,
    dry_run: bool,
) -> std::result::Result<(String, DocsPathMode), ConfigError> {
    // Try to get existing docs-path
    match syncdoc_core::config::get_docs_path(source_file.to_str().unwrap()) {
        Ok(path) => Ok((path, DocsPathMode::TomlConfig)),
        Err(_) => {
            // Need to add default docs-path to Cargo.toml
            if !dry_run {
                let source_dir = source_file.parent().ok_or_else(|| {
                    ConfigError::Other("Source file has no parent directory".to_string())
                })?;

                let manifest_dir = syncdoc_core::path_utils::find_manifest_dir(source_dir)
                    .ok_or_else(|| ConfigError::Other("Could not find Cargo.toml".to_string()))?;

                let cargo_toml_path = manifest_dir.join("Cargo.toml");

                // Read existing content
                let mut content = fs::read_to_string(&cargo_toml_path)?;

                // Check if syncdoc section exists
                if !content.contains("[package.metadata.syncdoc]") {
                    // Append the section
                    content.push_str("\n[package.metadata.syncdoc]\n");
                    content.push_str("docs-path = \"docs\"\n");

                    // Write back
                    fs::write(&cargo_toml_path, content)?;
                }
            }

            // We just created/will create TOML config
            Ok(("docs".to_string(), DocsPathMode::TomlConfig))
        }
    }
}

#[cfg(test)]
#[path = "tests/discover.rs"]
mod tests;
