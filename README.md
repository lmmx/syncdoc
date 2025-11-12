# syncdoc

[![crates.io](https://img.shields.io/crates/v/syncdoc.svg)](https://crates.io/crates/syncdoc)
[![documentation](https://docs.rs/syncdoc/badge.svg)](https://docs.rs/syncdoc)
[![MIT/Apache-2.0 licensed](https://img.shields.io/crates/l/syncdoc.svg)](./LICENSE)
[![pre-commit.ci status](https://results.pre-commit.ci/badge/github/lmmx/syncdoc/master.svg)](https://results.pre-commit.ci/latest/github/lmmx/syncdoc/master)
[![free of syn](https://img.shields.io/badge/free%20of-syn-hotpink)](https://github.com/fasterthanlime/free-of-syn)

syncdoc is a procedural macro that automatically injects documentation from external files into your Rust code, eliminating the need to manually maintain inline doc comments.

Use syncdoc when you want to keep documentation separate from implementation.

Stick with inline docs when you prefer co-location of docs and code.

## Motivation

When writing extensive documentation, keeping it inline can make code harder to read:
```rust
/// This is a very long doc comment
/// that spans many lines and makes
/// the actual function hard to see...
/// [more lines]
fn foo() { ... }

/// Another long doc comment
/// [many more lines]
fn bar() { ... }
```

syncdoc solves this by automatically pulling documentation from external files:

```rust
use syncdoc::omnidoc;

#[omnidoc(path = "../docs")]
mod my_functions {
    fn foo() { ... }  // Docs from ../docs/my_functions/foo.md
    fn bar() { ... }  // Docs from ../docs/my_functions/bar.md
}
```

## Installation

Add syncdoc to your `Cargo.toml`:
```toml
[dependencies]
syncdoc = "0.1"
```

## Setup

### `docs-path` (recommended)

To avoid specifying `path` in every attribute, add a default to your `Cargo.toml`
(it must be set one way or the other or the build will error).

```toml
[package.metadata.syncdoc]
docs-path = "docs"
```

Now you can use `#[omnidoc]` without arguments - syncdoc calculates the correct relative path automatically
(thanks to [this](https://docs.rs/proc-macro2/latest/proc_macro2/struct.Span.html#method.local_file) little trick specifically).

### `cfg-attr` (optional)

To generate `#[cfg_attr(doc, doc = "...")]` instead of `#[doc = "..."]` (meaning your docstrings will be `#[cfg(doc)]`-gated
(so `cargo doc` will generate them but `cargo build`/`check`/`test` will not), set the `cfg-attr` key to "doc" in your `Cargo.toml`.

```toml
[package.metadata.syncdoc]
cfg-attr = "doc"
```

See the _Build Configuration_ section below for more details.

### Migration

To automatically migrate code from doc comments to syncdoc `#[omnidoc]` attributes, install the CLI:

- pre-built binary: `cargo binstall syncdoc` (requires [cargo-binstall][cargo-binstall]),
- build from source: `cargo install syncdoc --features cli`

[cargo-binstall]: https://github.com/cargo-bins/cargo-binstall

Commit your code before running with `-c`/`--cut` or `-r`/`--rewrite` as they modify source files.

```help
Usage: syncdoc [OPTIONS] <SOURCE>

Migrate Rust documentation to external markdown files.

Arguments:
  <SOURCE>           Path to source directory to process (default: 'src')

Options:
  -d, --docs <dir>   Path to docs directory (default: 'docs' or from Cargo.toml if set)
  -c, --cut          Cut out doc comments from source files
  -r, --rewrite      Rewrite code with #[omnidoc] attributes
  -n, --dry-run      Preview changes without writing files
  -v, --verbose      Show verbose output
  -h, --help         Show this help message
```

#### Examples

- 'Sync' the docs dir with the docstrings in src/
```sh
syncdoc
```
- 'Cut' docstrings out of src/ as well as creating in docs/
```sh
syncdoc --cut # or -c
```
- 'Cut and paste' by replacing doc comments with omnidoc attributes
```sh
syncdoc --cut --add # or -ca
```
- Preview what would happen
```sh
syncdoc --cut --add --dry-run # or -can
```

### Usage

Apply the `#[omnidoc]` attribute to any module:

```rust
use syncdoc::omnidoc;

#[omnidoc(path = "../docs")]
mod my_functions {
    fn foo(x: i32) -> i32 {
        x * 2
    }

    fn bar(y: i32) -> i32 {
        y + 1
    }
}
```

This will look for documentation in:
- `../docs/my_functions/foo.md`
- `../docs/my_functions/bar.md`

> **Note**: you cannot use a proc macro on an external module,
> see [this](https://github.com/rust-lang/rust/issues/54727) tracking issue.
>
> A workaround to document an entire module is to inline the entire module (`mod mymodule { ... }`)
> then re-export it with `pub use mymodule::*;`. If you do, note that the name of the inner module is
> the name the macro will look for at the path.
>
> - See
>   [examples/demo_submodule](https://github.com/lmmx/syncdoc/tree/master/examples/demo_submodule)
>
> If that isn't to your liking, then just use it on impl blocks etc. and use a regular
> `syncdoc::omnidoc` attribute for individual items.

### Documenting Impl Blocks

syncdoc also works on impl blocks:
```rust
use syncdoc::omnidoc;

struct Calculator;

#[omnidoc(path = "../docs")]
impl Calculator {
    pub fn new() -> Self {
        Self
    }

    pub fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}
```

Documentation files:
- `../docs/Calculator/new.md`
- `../docs/Calculator/add.md`

### Single Function Documentation

You can also document individual functions. The difference between the syncdoc and omnidoc macros
comes down to whether or not you want to specify the exact path (typically you don't, so use omnidoc).

```rust
use syncdoc::{omnidoc, syncdoc};

#[omnidoc(path = "../docs")]
fn func1() {
    // -> ../docs/func1.md
    // = omnidoc(path) to root docs dir + submodule + fn name + .md
}

#[syncdoc(path = "../docs/new_func_name.md")]
fn func2() {
    // -> ../docs/new_func_name.md
    // = syncdoc(path) to single documentation file
}
```

### Documenting Structs and Enums

syncdoc automatically documents struct fields and enum variants:
```rust
use syncdoc::omnidoc;

#[omnidoc(path = "../docs")]
mod types {
    struct Config {
        port: u16,
        host: String,
    }

    enum Status {
        Active,
        Inactive,
        Error(String),
    }
}
```

Documentation files:
- `../docs/types/Config.md` - struct documentation
- `../docs/types/Config/port.md` - field documentation
- `../docs/types/Config/host.md` - field documentation
- `../docs/types/Status.md` - enum documentation
- `../docs/types/Status/Active.md` - variant documentation
- `../docs/types/Status/Inactive.md` - variant documentation
- `../docs/types/Status/Error.md` - variant documentation

## How It Works

syncdoc uses a procedural macro to inject `#[doc = include_str!("path")]` attributes before function definitions.

It uses `proc-macro2` (it's [free of `syn`](https://github.com/fasterthanlime/free-of-syn)!) to parse tokens rather than doing full AST creation.

### Implementation Details

The macro:

1. **Parses tokens** to find function definitions
2. **Constructs doc paths** based on module hierarchy and function names
3. **Injects doc attributes** using `include_str!` for compile-time validation
4. **Preserves existing attributes** and doesn't interfere with other macros

For examples of the generated output, see the [test snapshots](https://github.com/lmmx/syncdoc/tree/master/syncdoc-core/tests/snapshots) which show the exact documentation attributes injected for various code patterns.

### What Gets Documented

- Regular functions: `fn foo() { ... }`
- Generic functions: `fn foo<T>(x: T) { ... }`
- Methods in impl blocks: `impl MyStruct { fn method(&self) { ... } }`
- Trait default methods: `trait MyTrait { fn method() { ... } }`
- Struct fields: `struct Foo { field: i32 }`
- Enum variants: `enum Bar { Variant1, Variant2(i32) }`
- Type aliases: `type MyType = String;`
- Constants: `const X: i32 = 42;`
- Statics: `static Y: i32 = 42;`

## Build Configuration

For faster builds, you can configure syncdoc to only generate documentation during `cargo doc`:

| Example              | Macro invocation                     | TOML settings required | Generated attribute form                    |
| -------------------- | ------------------------------------ | ---------------------- | ------------------------------------------- |
| `demo_cfg_attr_call` | `#[cfg_attr(doc, syncdoc::omnidoc)]` | ❌ none                | `#[doc = include_str!(...)]`                |
| `demo_cfg_attr_toml` | `#[syncdoc::omnidoc]`                | ✅ `cfg-attr = "doc"`  | `#[cfg_attr(doc, doc = include_str!(...))]` |

**Option 1** gates the macro itself, at the call site. **Option 2** gates the generated attributes, configured in TOML (it can also be done at the call site,
but I'd recommended to do it in Cargo.toml to reduce the line noise in your code).

When using either approach, gate the `missing_docs` lint (if using it):

```rust
#![cfg_attr(doc, deny(missing_docs))]
```

## File Organization
```
my-project/
├── src/
│   ├── lib.rs
│   └── parser/
│       └── mod.rs       #[omnidoc(path = "../../docs")]
└── docs/
    └── parser/
        ├── parse_expr.md
        └── parse_stmt.md
```

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
