# 2025-01-11-4: Markdown File Writing

## Current State

- `syncdoc-core/src/token_processors.rs` implements `TokenProcessor` with `context: Vec<String>` tracking hierarchical path (token_processors.rs:8-11)
- `TokenProcessor::build_path()` constructs markdown paths from base_path + context + item name (token_processors.rs:195-203)
- `TokenProcessor::process_module_item()` matches on `ModuleItem` variants to handle different item types (token_processors.rs:37-72)

## Missing

- `syncdoc-migrate/src/write.rs` implements `DocExtraction` struct with fields: `markdown_path: PathBuf`, `content: String`, `source_location: String` (file:line format)
- `syncdoc-migrate/src/write.rs` implements `extract_all_docs(parsed: &ParsedFile, docs_root: &str) -> Vec<DocExtraction>` function
- `extract_all_docs()` creates `TokenProcessor` equivalent structure with empty context and docs_root as base_path
- Walk `parsed.content.items.0` vector, for each `ModuleItem` call recursive extraction
- `extract_item_docs(item: &ModuleItem, context: Vec<String>, base_path: &str) -> Vec<DocExtraction>` recursive function extracts from single item
- For Function items: call `extract::extract_doc_content()` on attrs, if Some build path via base_path/context/name.md, create `DocExtraction`
- For ImplBlock items: extract type name via `extract_type_name()` (reuse from token_processors.rs:430-438), push to context, recurse on impl body items
- For Module items: push module name to context, recurse on module body items
- For Trait items: push trait name to context, recurse on trait body items (only default methods have docs)
- For Enum items: extract enum doc if present, create extraction for enum itself, then iterate variants extracting variant docs with context = [enum_name]
- For Struct items: extract struct doc if present, create extraction for struct itself, then extract field docs (named fields only) with context = [struct_name]
- For TypeAlias, Const, Static items: extract doc content if present, create extraction with current context
- `extract_item_docs()` returns empty vector if item has no body to recurse into and no doc attrs
- `syncdoc-migrate/src/write.rs` implements `write_extractions(extractions: &[DocExtraction], dry_run: bool) -> Result<WriteReport, std::io::Error>` function
- `write_extractions()` groups by parent directory, calls `std::fs::create_dir_all()` once per directory
- For each extraction, write content to markdown_path via `std::fs::write()`
- If dry_run is true, skip actual writes but still validate paths and report what would be written
- `WriteReport` struct tracks `files_written: usize`, `files_skipped: usize`, `errors: Vec<String>`
- Return error only for fatal issues, collect non-fatal errors in `errors` vector
- Unit test `test_extract_nested_module_paths()` verifies context stacking produces correct paths like docs/outer/inner/func.md
- Unit test `test_extract_impl_method_paths()` verifies impl methods get docs/TypeName/method.md paths
- Unit test `test_extract_enum_variant_paths()` verifies variants get docs/EnumName/VariantName.md paths
- Integration test `test_write_creates_directories()` uses tempfile to verify directory creation and file writing
