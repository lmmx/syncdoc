An example of how to use this crate to document an enum with docstrings generated to look like:

```rust
#[cfg_attr(doc, doc = ...)]
```

Such that only docs builds have docstrings. To get this, set the `cfg-attr` key for the syncdoc package metadata.

- Note: If you want all builds to have docstrings (i.e. to generate `#[doc = ...]`), see the other example:
  [demo_cfg_attr_call][demo_cfg_attr_call], and do not set the `cfg-attr` key on the syncdoc package metadata in
  your Cargo.toml.

[demo_cfg_attr_call]: https://github.com/lmmx/syncdoc/blob/master/examples/demo_cfg_attr_call

If using cargo's `missing_docs` lint level (recommended!) then you should gate it behind `cfg_attr(doc)`
and ensure the rustdoc build is part of your development checks (pre-commit hooks, etc).

- `cargo check` won't catch it once gated behind `doc`
- `cargo doc` will catch it as a lint error

This should encourage more use of the `rustdoc` namespaced lints, check them out [here][rustdoc-lints].

[rustdoc-lints]: https://doc.rust-lang.org/rustdoc/lints.html

This crate shows how to do this in `src/lib.rs`:

```rust
#![cfg_attr(doc, deny(missing_docs))]
```

This approach will make your builds/tests slightly faster.

```sh
cargo doc
```

```rust
 Documenting demo_cfg_attr_call v0.1.2 (/home/louis/dev/syncdoc/examples/demo_cfg_attr_call)
error: missing documentation for an enum
 --> examples/demo_cfg_attr_call/src/lib.rs:4:1
  |
4 | pub enum Foo {
  | ^^^^^^^^^^^^
  |
note: the lint level is defined here
 --> examples/demo_cfg_attr_call/src/lib.rs:2:23
  |
2 | #![cfg_attr(doc, deny(missing_docs))]
  |                       ^^^^^^^^^^^^
```
