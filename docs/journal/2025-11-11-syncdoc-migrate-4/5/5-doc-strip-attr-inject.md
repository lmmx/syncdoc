# 2025-01-11-5: Doc Stripping and Attribute Injection

## Current State

- `syncdoc-core/src/doc_injector.rs` implements `inject_doc_attr(doc_path: String, cfg_attr: Option<String>, item: TokenStream) -> TokenStream` that prepends doc attribute
- `inject_doc_attr()` uses `cfg_attr` parameter to decide between `#[cfg_attr(doc, doc = ...)]` and `#[doc = ...]` forms
- All item parsers preserve non-doc attributes in `Option<AttributeList>` field
- `quote!` macro available for reconstructing token streams

## Implementation

### Core Functions

- `syncdoc-migrate/src/rewrite.rs` implements `strip_doc_attrs(item: TokenStream) -> TokenStream` function
- `strip_doc_attrs()` parses token stream looking for `#[doc` attribute patterns
- Collects all tokens into vector, iterates with lookahead to detect attribute start (`#` followed by `[`)
- When attribute detected, parses into group, checks if contains `doc` identifier via `is_doc_attribute()` helper
- If doc attribute, skips entire attribute group (consumes both `#` and `[...]` tokens)
- If non-doc attribute, preserves all tokens in output stream
- Reconstructs `TokenStream` from filtered tokens via direct token collection
- `strip_doc_attrs_recursive()` variant recursively strips doc attrs from nested items (e.g., fields inside structs)
- `is_doc_attribute()` helper checks for `doc = "..."`, `doc(hidden)`, and `cfg_attr(doc, ...)` patterns

### Attribute Injection

- `syncdoc-migrate/src/rewrite.rs` implements `inject_omnidoc_attr(item: TokenStream, docs_root: &str) -> TokenStream` function
- `inject_omnidoc_attr()` constructs `#[omnidoc(path = "docs_root")]` attribute via `quote!` with docs root directory path
- Inserts attribute after visibility tokens but before other attributes (handles `pub`, `pub(crate)`, etc.)
- **Always uses `#[omnidoc]`** - path parameter is the docs root directory, not the specific file path
- omnidoc automatically determines the correct markdown file path based on item name
- For containers (modules, impls, traits, structs with fields, enums with variants): omnidoc documents parent and all children
- For leaf items (functions, type aliases, consts, statics): omnidoc acts like syncdoc and documents just that item
- No `needs_omnidoc()` or `inject_syncdoc_attr()` functions needed - omnidoc handles all cases

### File Rewriting

- `syncdoc-migrate/src/rewrite.rs` implements `rewrite_file(parsed: &ParsedFile, docs_root: &str, strip: bool, annotate: bool) -> Option<String>` function
- If neither strip nor annotate, returns `None` to signal no rewrite needed
- If strip, applies `strip_doc_attrs_recursive()` to each item's tokens (handles nested doc attrs)
- If annotate, injects `#[omnidoc(path = docs_root)]` on all named items
- Reconstructs entire file token stream preserving structure
- Returns formatted string via `TokenStream::to_string()`

### Tests

- Unit test `test_strip_preserves_non_doc()` verifies `#[derive(Debug)]` and `#[cfg(test)]` preserved after stripping doc attrs
- Unit test `test_strip_removes_all_doc()` verifies multiple consecutive doc attrs all removed
- Unit test `test_strip_removes_cfg_attr_doc()` verifies `#[cfg_attr(doc, ...)]` forms are stripped
- Unit test `test_inject_omnidoc_after_visibility()` verifies attribute placed after `pub` but before `#[derive]`
- Unit test `test_inject_omnidoc_before_derive()` verifies omnidoc comes before other attributes
- Unit test `test_inject_omnidoc_no_visibility()` verifies injection works without visibility modifiers
- Unit test `test_rewrite_roundtrip()` verifies stripping then re-injecting produces valid compilable code
- Unit test `test_rewrite_none_when_no_ops()` verifies `None` returned when neither strip nor annotate requested
- Integration snapshot tests in `tests/rewrite_integration.rs` verify:
  - Doc stripping on functions, structs, enums, modules
  - Omnidoc annotation on all item types
  - Combined strip + annotate operations
  - Preservation of other attributes during operations
  - Handling of nested documentation (struct fields, enum variants)
  - Unit structs, tuple structs, and empty structs
  - Complex mixed files with multiple item types
