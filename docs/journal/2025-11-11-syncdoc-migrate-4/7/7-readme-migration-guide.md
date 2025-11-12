# 2025-01-11-7: README Migration Guide

## Current State

- `README.md` documents basic usage with `#[omnidoc]` and `#[syncdoc]` attributes (README.md:35-85)
- `README.md` includes installation section showing `cargo add syncdoc` (README.md:27-30)
- `README.md` has "How It Works" section explaining proc macro behavior (README.md:104-123)

## Missing

- `README.md` adds "Migration Guide" section after "Installation" section before "Usage"
- Migration Guide section starts with overview: "If you have existing inline documentation, syncdoc provides a CLI tool to automatically migrate to external markdown files"
- Subsection "Installing the Migration Tool" shows: `cargo install syncdoc --features cli` or `cargo binstall syncdoc --features cli` for faster installation
- Subsection "Basic Migration" shows command: `syncdoc --source src --docs docs` with explanation that this extracts docs to markdown files without modifying source
- Explains that tool reads or creates `[package.metadata.syncdoc]` metadata in Cargo.toml with `docs-path = "docs"` entry
- Subsection "Migration with Cleanup" shows: `syncdoc --source src --docs docs --strip-docs --annotate` with explanation this removes inline docs and adds syncdoc attributes
- Lists what `--strip-docs` does: removes all `#[doc = "..."]` attributes from source code
- Lists what `--annotate` does: adds `#[omnidoc]` to modules/impls/traits/enums/structs and `#[syncdoc]` to standalone functions
- Subsection "Migration Workflow" provides step-by-step: 1) Run with `--dry-run` to preview, 2) Run without dry-run to extract, 3) Optionally run with `--strip-docs --annotate`, 4) Test with `cargo doc` to verify, 5) Commit changes
- Warning box states: "Always commit your code before running migration with `--strip-docs` as it modifies source files"
- Subsection "What Gets Migrated" lists same items as "What Gets Documented" section: functions, methods, struct fields, enum variants, type aliases, consts, statics
- Subsection "Migration Limitations" notes: tool cannot migrate docs from external modules (due to Rust proc macro restrictions), suggests inlining module with re-export pattern if needed
- Code example showing before/after of migrated code: before shows inline docs, after shows `#[omnidoc]` with reference to markdown files
- Add link from "Motivation" section to Migration Guide: "See the Migration Guide below for automatically converting existing inline docs"
