# 2025-01-11-2: Doc Attribute Extraction



## Current State

- `syncdoc-core/src/parse/mod.rs` defines `AttributeList` as `Many<Attribute>` (parse/mod.rs:18)
- `Attribute` type from `unsynn` provides token access via `ToTokens` trait
- All item parsers (`FnSig`, `ImplBlockSig`, etc) include `Option<AttributeList>` field for attributes

## Missing

- `syncdoc-migrate/src/extract.rs` implements `extract_doc_content(attrs: &Option<AttributeList>) -> Option<String>` function
- `extract_doc_content()` returns `None` if `attrs` is `None` or contains no doc attributes
- `extract_doc_content()` iterates through `attrs.0` vector checking each `Attribute`
- For each attribute, convert to `TokenStream` via `ToTokens`, check if it starts with `#[doc` token sequence
- Extract string content between `doc = "` and closing `"` from each doc attribute's token stream
- Concatenate all doc strings with single newline separator, maintaining order from source
- Trim leading/trailing whitespace from concatenated result but preserve internal structure
- Return `Some(content)` only if at least one doc attribute found, otherwise `None`
- `syncdoc-migrate/src/extract.rs` implements `has_doc_attrs(attrs: &Option<AttributeList>) -> bool` helper that returns true if any doc attributes present
- Unit test `test_extract_empty()` verifies `None` returned for no attributes
- Unit test `test_extract_single()` verifies single doc comment extracted correctly
- Unit test `test_extract_multiple()` verifies multiple doc comments concatenated with newlines
- Unit test `test_extract_preserves_formatting()` verifies markdown formatting inside docs preserved
- Unit test `test_extract_ignores_non_doc()` verifies derive/cfg attributes don't affect extraction
