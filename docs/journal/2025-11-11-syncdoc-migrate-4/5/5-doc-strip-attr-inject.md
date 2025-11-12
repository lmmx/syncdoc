# 2025-01-11-5: Doc Stripping and Attribute Injection

## Current State

- `syncdoc-core/src/doc_injector.rs` implements `inject_doc_attr(doc_path: String, item: TokenStream) -> TokenStream` that prepends doc attribute (doc_injector.rs:125-137)
- `inject_doc_attr()` checks `cfg!(feature = "cfg-attr-doc")` to decide between `#[cfg_attr(doc, doc = ...)]` and `#[doc = ...]` forms
- All item parsers preserve non-doc attributes in `Option<AttributeList>` field
- `quote!` macro available for reconstructing token streams

## Missing

- `syncdoc-migrate/src/rewrite.rs` implements `strip_doc_attrs(item: TokenStream) -> TokenStream` function
- `strip_doc_attrs()` parses token stream looking for `#[doc` attribute patterns
- Collect all tokens into vector, iterate with lookahead to detect attribute start (`#` followed by `[`)
- When attribute detected, parse into group, check if contains `doc` identifier
- If doc attribute, skip entire attribute group (consume tokens until matching `]`)
- If non-doc attribute, preserve all tokens in output stream
- Reconstruct `TokenStream` from filtered tokens via `quote!` or direct token collection
- `syncdoc-migrate/src/rewrite.rs` implements `needs_omnidoc(item: &ModuleItem) -> bool` function determining if item should get `#[omnidoc]`
- `needs_omnidoc()` returns true for: Module, ImplBlock, Trait (items that contain multiple documentable children)
- `needs_omnidoc()` returns false for: Function (standalone, gets `#[syncdoc]` instead), TypeAlias, Const, Static (no children)
- `needs_omnidoc()` returns true for: Enum, Struct if they have variants/fields that will be documented
- `syncdoc-migrate/src/rewrite.rs` implements `inject_syncdoc_attr(item: TokenStream, doc_path: &str) -> TokenStream` function
- `inject_syncdoc_attr()` constructs `#[syncdoc(path = "...")]` attribute via `quote!` with provided path
- Insert attribute after visibility tokens but before other attributes (parse for `pub`, `pub(crate)` etc)
- `syncdoc-migrate/src/rewrite.rs` implements `inject_omnidoc_attr(item: TokenStream, doc_path: &str) -> TokenStream` function
- `inject_omnidoc_attr()` constructs `#[omnidoc(path = "...")]` attribute, places same as `inject_syncdoc_attr()`
- `syncdoc-migrate/src/rewrite.rs` implements `rewrite_file(parsed: &ParsedFile, docs_root: &str, strip: bool, annotate: bool) -> Option<String>` function
- If neither strip nor annotate, return `None` to signal no rewrite needed
- If strip, walk all items recursively, apply `strip_doc_attrs()` to each item's tokens
- If annotate, walk all items, check `needs_omnidoc()`, inject appropriate attribute with path relative to docs_root
- Reconstruct entire file token stream preserving structure and formatting
- Format via `ToTokens::tokens_to_string`
- Unit test `test_strip_preserves_non_doc()` verifies `#[derive(Debug)]` and `#[cfg(test)]` preserved after stripping doc attrs
- Unit test `test_strip_removes_all_doc()` verifies multiple consecutive doc attrs all removed
- Unit test `test_inject_omnidoc_after_visibility()` verifies attribute placed after `pub` but before `#[derive]`
- Unit test `test_needs_omnidoc_logic()` verifies correct true/false for each `ModuleItem` variant
- Integration test `test_rewrite_roundtrip()` verifies stripping then re-injecting produces valid compilable code
