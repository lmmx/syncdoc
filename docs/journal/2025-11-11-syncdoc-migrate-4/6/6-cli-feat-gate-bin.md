# 2025-01-11-6: CLI Feature Gate and Binary

## Current State

- `syncdoc/Cargo.toml` defines workspace member "syncdoc" with proc-macro = true (syncdoc/Cargo.toml:18)
- `syncdoc/Cargo.toml` has features section with "cfg-attr-doc" feature (syncdoc/Cargo.toml:14-15)
- Workspace includes examples with pattern of CLI feature gates (textum/Cargo.toml:1-4 shows `[[bin]]` with required-features)
- `textum/src/cli.rs` shows pattern: feature-gated `pub mod inner` with implementation, fallback mod prints error message (textum/src/cli.rs:13-79)

## Missing

- `syncdoc/Cargo.toml` adds `cli` feature: `cli = ["syncdoc-migrate"]` in features section
- `syncdoc/Cargo.toml` adds optional dependency: `syncdoc-migrate = { path = "../syncdoc-migrate", optional = true }` in dependencies section
- `syncdoc/Cargo.toml` adds binary definition: `[[bin]]` section with `name = "syncdoc"`, `path = "src/cli.rs"`, `required-features = ["cli"]`
- `syncdoc/src/cli.rs` created with feature-gated module structure matching textum pattern
- `syncdoc/src/cli.rs` contains `#[cfg(feature = "cli")] pub mod inner` block
- `inner` module declares `use syncdoc_migrate;` and implements `pub fn main() -> std::io::Result<()>`
- `main()` parses CLI args for: `--source <path>`, `--docs <path>`, `--strip-docs`, `--annotate`, `--dry-run`, `--verbose` flags
- Use simple arg parsing (avoid clap dependency, just iterate `std::env::args()` and match patterns)
- `main()` calls `syncdoc_migrate::discover::discover_rust_files()` to find source files
- For each file, call `syncdoc_migrate::discover::parse_file()`, skip with warning if parse fails
- Call `syncdoc_migrate::discover::get_or_create_docs_path()` to ensure Cargo.toml metadata present
- Call `syncdoc_migrate::write::extract_all_docs()` to get extractions for file
- Call `syncdoc_migrate::write::write_extractions()` with dry_run flag
- If `--strip-docs` or `--annotate` specified, call `syncdoc_migrate::rewrite::rewrite_file()` and write result back to source
- Print summary: files processed, docs extracted, files rewritten, any errors
- `syncdoc/src/cli.rs` contains `#[cfg(not(feature = "cli"))] pub mod inner` block
- Fallback inner module implements `pub fn main()` that prints "Please build with the cli feature: cargo install syncdoc --features cli" and exits with code 1
- `syncdoc/src/cli.rs` ends with `pub use inner::main;` to export main function
- `syncdoc-migrate/Cargo.toml` adds `prettyplease` dependency for code formatting in rewrite phase
- Integration test in `syncdoc-migrate/tests/cli_integration.rs` creates temp project with documented code, runs migration, verifies markdown files created and source optionally modified
