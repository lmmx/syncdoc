# syncdoc

[![MIT/Apache-2.0 licensed](https://img.shields.io/crates/l/syncdoc.svg)](./LICENSE)
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
#[syncdoc(path = "../docs")]
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

## Usage

### Basic Usage

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

You can also document individual functions:
```rust
use syncdoc::syncdoc;

#[syncdoc(path = "../docs/special_function.md")]
fn special_function() {
    // Implementation
}
```

## How It Works

syncdoc uses a procedural macro to inject `#[doc = include_str!("path")]` attributes before function definitions.

It uses `proc-macro2` (it's [free of `syn`](https://github.com/fasterthanlime/free-of-syn)!) to parse tokens rather than doing full AST creation.

### Implementation Details

The macro:

1. **Parses tokens** to find function definitions
2. **Constructs doc paths** based on module hierarchy and function names
3. **Injects doc attributes** using `include_str!` for compile-time validation
4. **Preserves existing attributes** and doesn't interfere with other macros

### What Gets Documented

- Regular functions: `fn foo() { ... }`
- Generic functions: `fn foo<T>(x: T) { ... }`
- Methods in impl blocks: `impl MyStruct { fn method(&self) { ... } }`
- Trait default methods: `trait MyTrait { fn method() { ... } }`

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
