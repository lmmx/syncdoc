# 2025-01-11-3: File Discovery and Module Parsing



## Current State

- `syncdoc-core/src/parse.rs` defines `ModuleContent` parser that accepts `TokenStream` and returns parsed items (parse.rs:434-438)
- `ModuleContent` contains `Many<Cons<ModuleItem, Eol>>` representing all items in module
- `ModuleItem` enum covers all documentable items: Function, ImplBlock, Module, Trait, Enum, Struct, TypeAlias, Const, Static, Other
- `syncdoc-core/src/config.rs` provides `get_docs_path(source_file: &str)` that reads Cargo.toml metadata

## Missing

- `syncdoc-migrate/src/discover.rs` implements `discover_rust_files(source_dir: &Path) -> Result<Vec<PathBuf>, std::io::Error>` function
- `discover_rust_files()` uses `std::fs::read_dir()` recursively to walk directory tree
- Filter entries to only `.rs` files via `path.extension() == Some("rs")`
- Return sorted vector of absolute paths for deterministic processing order
- `syncdoc-migrate/src/discover.rs` implements `parse_file(path: &Path) -> Result<ParsedFile, ParseError>` function
- `parse_file()` reads file content via `std::fs::read_to_string()`
- Convert string to `TokenStream` via `TokenStream::from_str()` (requires `use std::str::FromStr`)
- Parse token stream into `ModuleContent` using `input.into_token_iter().parse::<ModuleContent>()`
- `ParsedFile` struct contains `path: PathBuf`, `content: ModuleContent`, `original_source: String` fields
- `ParseError` enum includes `IoError(std::io::Error)` and `ParseFailed(String)` variants
- Return `Err(ParseError::ParseFailed)` if `parse::<ModuleContent>()` fails, allowing caller to skip unparseable files
- `syncdoc-migrate/src/discover.rs` implements `get_or_create_docs_path(source_file: &Path) -> Result<String, ConfigError>` function
- `get_or_create_docs_path()` first calls `syncdoc_core::config::get_docs_path()` to check existing metadata
- If `get_docs_path()` returns error about missing metadata, read Cargo.toml, append `[package.metadata.syncdoc]\ndocs-path = "docs"`, write back
- Return "docs" as default path after writing metadata
- Unit test `test_discover_finds_rs_files()` creates temp dir with `.rs` and `.txt` files, verifies only `.rs` returned
- Unit test `test_parse_valid_module()` verifies parsing succeeds for simple module with functions
- Unit test `test_parse_invalid_returns_error()` verifies malformed Rust returns `ParseError::ParseFailed`
