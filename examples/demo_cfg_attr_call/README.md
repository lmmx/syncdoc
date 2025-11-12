An example of how to use this crate to document an enum with the attribute macro gated with

```rust
#[cfg_attr(doc, syncdoc::omnidoc)]
```

Such that it will only be compiled for docs builds.

- Note: if you want to always run the codegen, but have the generated `#[doc]` attributes themselves
  be only present in docs builds, use the `cfg-attr` flag in the syncdoc Cargo TOML metadata section.
  See the other example: [demo_cfg_attr_toml][demo_cfg_attr_toml], and set the `cfg-attr` key on the
  `[package.metadata.syncdoc]` section in your Cargo.toml.

[demo_cfg_attr_toml]: https://github.com/lmmx/syncdoc/blob/master/examples/demo_cfg_attr_toml

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
 Documenting demo_cfg_attr v0.1.2 (/home/louis/dev/syncdoc/examples/demo_cfg_attr)
error: missing documentation for an enum
 --> examples/demo_cfg_attr/src/lib.rs:4:1
  |
4 | pub enum Foo {
  | ^^^^^^^^^^^^
  |
note: the lint level is defined here
 --> examples/demo_cfg_attr/src/lib.rs:2:23
  |
2 | #![cfg_attr(doc, deny(missing_docs))]
  |                       ^^^^^^^^^^^^
```
