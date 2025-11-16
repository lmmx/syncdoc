/// Configuration mode for documentation paths
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocsPathMode {
    /// Path configured in Cargo.toml [package.metadata.syncdoc]
    /// Macros will be called without path arguments: `#[omnidoc]`, `module_doc!()`
    TomlConfig,

    /// Path specified inline in each macro call
    /// Macros will include path: `#[omnidoc(path = "docs")]`, `module_doc!(path = "docs")`
    InlinePaths,
}
