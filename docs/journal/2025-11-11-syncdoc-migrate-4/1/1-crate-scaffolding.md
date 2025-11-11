# 2025-01-11-1: syncdoc-migrate Crate Scaffolding

## Missing

- `syncdoc-migrate/` directory at workspace root alongside existing `syncdoc/` and `syncdoc-core/` directories
- `syncdoc-migrate/Cargo.toml` with package metadata matching workspace conventions: edition, license, repository from workspace, version "0.1.4", description "CLI tool for migrating inline documentation to syncdoc format"
- `syncdoc-migrate/Cargo.toml` depends on `syncdoc-core = { workspace = true }` for reusing parsers and path construction
- `syncdoc-migrate/Cargo.toml` depends on `proc-macro2 = { workspace = true }` for token stream manipulation
- `syncdoc-migrate/Cargo.toml` depends on `quote = { workspace = true }` for token reconstruction
- `syncdoc-migrate/Cargo.toml` depends on `ropey = { workspace = true }` for rope-based text editing
- `syncdoc-migrate/src/lib.rs` with module declarations: `mod extract;`, `mod discover;`, `mod write;`, `mod rewrite;`, `mod report;`
- `syncdoc-migrate/src/lib.rs` exports public `migrate()` function that will orchestrate migration process
- `syncdoc-migrate/src/extract.rs` empty module with `// TODO: implement extract_doc_content()`
- `syncdoc-migrate/src/discover.rs` empty module with `// TODO: implement file discovery`
- `syncdoc-migrate/src/write.rs` empty module with `// TODO: implement markdown writing`
- `syncdoc-migrate/src/rewrite.rs` empty module with `// TODO: implement doc stripping and attribute injection`
- `syncdoc-migrate/src/report.rs` empty module with `// TODO: implement migration report`
- Workspace `Cargo.toml` members array includes "syncdoc-migrate" entry after "syncdoc-core"
- `syncdoc-migrate/Cargo.toml` includes dev-dependency `tempfile.workspace = true` for integration tests
- `syncdoc-migrate/Cargo.toml` includes dev-dependency `insta.workspace = true` for snapshot tests
