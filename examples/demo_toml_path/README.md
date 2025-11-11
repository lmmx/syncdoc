An example of how to use this crate to document Rust code
without specifying a path at the call site
(instead taking the docs path root from the Cargo.toml:

```toml
[package.metadata.syncdoc]
docs-path = "docs"
```
