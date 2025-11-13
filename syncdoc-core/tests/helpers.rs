// syncdoc-core/tests/helpers.rs
#![cfg(test)]
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

pub struct TestCrate {
    _temp_dir: TempDir,
    root: PathBuf,
}

impl TestCrate {
    pub fn new(name: &str) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Should have parent")
            .to_path_buf();

        let cargo_toml = format!(
            r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
syncdoc = {{ path = "{}" }}

[package.metadata.syncdoc]
docs-path = "docs"
"#,
            name,
            workspace_root.join("syncdoc").display()
        );

        fs::write(root.join("Cargo.toml"), cargo_toml).unwrap();
        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir_all(root.join("docs/lib")).unwrap();

        Self {
            _temp_dir: temp_dir,
            root,
        }
    }

    pub fn write_lib(&self, code: &str) {
        let full_content = format!(
            r#"
#![doc = include_str!("../docs/lib.md")]

use syncdoc::omnidoc;

#[omnidoc(path = "docs")]
{}"#,
            code
        );
        fs::write(self.root.join("src/lib.rs"), full_content).unwrap();

        // After writing, add dummy type definitions
        self.create_dummy_types(code);
    }

    pub fn write_doc(&self, relative_path: &str, content: &str) {
        let full_path = self.root.join("docs").join(relative_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full_path, content).unwrap();
    }

    /// Automatically creates all expected doc files based on the code structure
    pub fn auto_create_docs(&self, code: &str) {
        // Always create the lib.md
        self.write_doc("lib.md", "Test library");

        // Parse the code for items and create corresponding docs
        for line in code.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            // Match functions - look for "fn " anywhere in the line
            // This catches: fn, pub fn, async fn, unsafe fn, const fn, pub async fn, etc.
            if let Some(fn_pos) = trimmed.find("fn ") {
                // Make sure it's actually a function definition, not a comment or string
                // Check that "fn " is preceded by whitespace or is at the start
                let before_fn = if fn_pos > 0 { &trimmed[..fn_pos] } else { "" };

                // Valid if it's at start, or preceded by keywords/whitespace
                let is_valid_fn =
                    fn_pos == 0 || before_fn.ends_with(' ') || before_fn.ends_with('\t');

                if is_valid_fn {
                    let after_fn = &trimmed[fn_pos + 3..]; // Skip "fn "
                    if let Some(name_end) = after_fn.find(|c: char| c == '(' || c == '<') {
                        let clean_name = after_fn[..name_end].trim();
                        if !clean_name.is_empty() && !clean_name.contains(' ') {
                            eprintln!("Creating doc for function: {}", clean_name);
                            self.write_doc(
                                &format!("lib/{}.md", clean_name),
                                &format!("Documentation for {}", clean_name),
                            );
                        }
                    }
                }
            }

            // Match: struct Name
            if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
                let struct_start = trimmed
                    .strip_prefix("pub struct ")
                    .or_else(|| trimmed.strip_prefix("struct "))
                    .unwrap();

                if let Some(name) = struct_start
                    .split(|c: char| c.is_whitespace() || c == '{' || c == ';' || c == '<')
                    .next()
                {
                    let clean_name = name.trim();
                    if !clean_name.is_empty() {
                        eprintln!("Creating doc for struct: {}", clean_name);
                        self.write_doc(
                            &format!("lib/{}.md", clean_name),
                            &format!("Documentation for {}", clean_name),
                        );
                    }
                }
            }

            // Match: enum Name
            if trimmed.starts_with("enum ") || trimmed.starts_with("pub enum ") {
                let enum_start = trimmed
                    .strip_prefix("pub enum ")
                    .or_else(|| trimmed.strip_prefix("enum "))
                    .unwrap();

                if let Some(name) = enum_start
                    .split(|c: char| c.is_whitespace() || c == '{' || c == '<')
                    .next()
                {
                    let clean_name = name.trim();
                    if !clean_name.is_empty() {
                        eprintln!("Creating doc for enum: {}", clean_name);
                        self.write_doc(
                            &format!("lib/{}.md", clean_name),
                            &format!("Documentation for {}", clean_name),
                        );
                    }
                }
            }

            // Match: const NAME
            if trimmed.starts_with("const ") || trimmed.starts_with("pub const ") {
                let const_start = trimmed
                    .strip_prefix("pub const ")
                    .or_else(|| trimmed.strip_prefix("const "))
                    .unwrap();

                if let Some(name) = const_start.split(':').next() {
                    let clean_name = name.trim();
                    if !clean_name.is_empty() {
                        eprintln!("Creating doc for const: {}", clean_name);
                        self.write_doc(
                            &format!("lib/{}.md", clean_name),
                            &format!("Documentation for {}", clean_name),
                        );
                    }
                }
            }
        }

        // Special handling for impl blocks and modules (more complex parsing)
        self.handle_impl_blocks(code);
        self.handle_modules(code);
    }

    fn handle_impl_blocks(&self, code: &str) {
        let mut in_impl = false;
        let mut impl_name = String::new();
        let mut brace_depth = 0;

        for line in code.lines() {
            let trimmed = line.trim();

            if (trimmed.starts_with("impl ") || trimmed.starts_with("impl<")) && !in_impl {
                in_impl = true;
                brace_depth = 0;

                // Extract type name (handle both "impl Type" and "impl Trait for Type")
                let impl_part = trimmed.split('{').next().unwrap_or("");

                if let Some(for_pos) = impl_part.find(" for ") {
                    // "impl Trait for Type" - use Type
                    let after_for = &impl_part[for_pos + 5..];
                    if let Some(name) = after_for
                        .split(|c: char| c.is_whitespace() || c == '<')
                        .next()
                    {
                        impl_name = name.trim().to_string();
                    }
                } else {
                    // "impl Type" or "impl<T> Type" - extract the type name
                    let parts: Vec<&str> = impl_part.split_whitespace().collect();
                    if let Some(last) = parts.last() {
                        impl_name = last.trim().split('<').next().unwrap_or(last).to_string();
                    }
                }

                if !impl_name.is_empty() {
                    fs::create_dir_all(self.root.join("docs/lib").join(&impl_name)).ok();
                }
            }

            if in_impl {
                // Count braces
                brace_depth += trimmed.matches('{').count();
                brace_depth -= trimmed.matches('}').count();

                // Look for function definitions
                if let Some(fn_pos) = trimmed.find("fn ") {
                    let after_fn = &trimmed[fn_pos + 3..];
                    if let Some(name_end) = after_fn.find(|c: char| c == '(' || c == '<') {
                        let clean_name = after_fn[..name_end].trim();
                        if !clean_name.is_empty()
                            && !impl_name.is_empty()
                            && !clean_name.contains(' ')
                        {
                            eprintln!("Creating doc for method: {}::{}", impl_name, clean_name);
                            self.write_doc(
                                &format!("lib/{}/{}.md", impl_name, clean_name),
                                &format!("Documentation for {}::{}", impl_name, clean_name),
                            );
                        }
                    }
                }

                // Exit impl block when braces are balanced
                if brace_depth == 0 && trimmed.ends_with('}') {
                    in_impl = false;
                    impl_name.clear();
                }
            }
        }
    }

    fn handle_modules(&self, code: &str) {
        let mut in_mod = false;
        let mut mod_name = String::new();
        let mut depth = 0;

        for line in code.lines() {
            let trimmed = line.trim();

            if (trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ")) && !in_mod {
                let mod_start = trimmed
                    .strip_prefix("pub mod ")
                    .or_else(|| trimmed.strip_prefix("mod "))
                    .unwrap();

                if let Some(name) = mod_start.split('{').next() {
                    mod_name = name.trim().to_string();
                    in_mod = true;
                    depth = 0;
                    fs::create_dir_all(self.root.join("docs/lib").join(&mod_name)).ok();
                    eprintln!("Creating doc for module: {}", mod_name);
                    self.write_doc(
                        &format!("lib/{}.md", mod_name),
                        &format!("Documentation for {}", mod_name),
                    );
                }
            }

            if in_mod {
                if trimmed.contains('{') {
                    depth += trimmed.matches('{').count();
                }

                if let Some(fn_pos) = trimmed.find("fn ") {
                    let after_fn = &trimmed[fn_pos + 3..];
                    if let Some(name_end) = after_fn.find(|c: char| c == '(' || c == '<') {
                        let clean_name = after_fn[..name_end].trim();
                        if !clean_name.is_empty() && !clean_name.contains(' ') {
                            eprintln!(
                                "Creating doc for module function: {}::{}",
                                mod_name, clean_name
                            );
                            self.write_doc(
                                &format!("lib/{}/{}.md", mod_name, clean_name),
                                &format!("Documentation for {}::{}", mod_name, clean_name),
                            );
                        }
                    }
                }

                if trimmed.contains('}') {
                    depth -= trimmed.matches('}').count();
                    if depth == 0 {
                        in_mod = false;
                    }
                }
            }
        }
    }

    pub fn cargo_check(&self) -> (bool, String) {
        let output = Command::new("cargo")
            .args(&["check", "--quiet"])
            .current_dir(&self.root)
            .output()
            .expect("Failed to run cargo check");

        let success = output.status.success();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        (success, stderr)
    }

    /// Gets the expanded/documented source code for snapshotting
    pub fn get_expanded_lib(&self) -> Option<String> {
        // Run rustfmt on the lib.rs to get formatted output
        let lib_path = self.root.join("src/lib.rs");
        fs::read_to_string(lib_path).ok()
    }

    /// Creates dummy type definitions for impl blocks to compile
    fn create_dummy_types(&self, code: &str) {
        let mut types_to_define = Vec::new();

        // Find all impl blocks and extract the type names
        for line in code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("impl ") || trimmed.starts_with("impl<") {
                let impl_part = trimmed.split('{').next().unwrap_or("");

                if let Some(for_pos) = impl_part.find(" for ") {
                    // "impl Trait for Type" - need both
                    let trait_part = &impl_part[4..for_pos].trim();
                    let type_part = &impl_part[for_pos + 5..].trim();

                    // Extract trait name
                    let trait_name = trait_part
                        .split_whitespace()
                        .filter(|s| *s != "unsafe")
                        .last()
                        .unwrap_or("")
                        .split('<')
                        .next()
                        .unwrap_or("")
                        .trim();
                    if !trait_name.is_empty() && trait_name.chars().next().unwrap().is_uppercase() {
                        types_to_define.push(format!("trait {} {{}}", trait_name));
                    }

                    // Extract type name
                    let type_name = type_part
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .split('<')
                        .next()
                        .unwrap_or("")
                        .trim();
                    if !type_name.is_empty() && type_name.chars().next().unwrap().is_uppercase() {
                        types_to_define.push(format!("struct {};", type_name));
                    }
                } else {
                    // "impl Type" or "impl<T> Type"
                    let parts: Vec<&str> = impl_part.split_whitespace().collect();
                    if let Some(last) = parts.last() {
                        let type_name = last
                            .split('<')
                            .next()
                            .unwrap_or(last)
                            .split('{')
                            .next()
                            .unwrap_or("")
                            .trim();
                        if !type_name.is_empty()
                            && type_name
                                .chars()
                                .next()
                                .map(|c| c.is_uppercase())
                                .unwrap_or(false)
                        {
                            // Check if it has generics in the original
                            if impl_part.contains('<') && !impl_part.starts_with("impl<") {
                                // Type has generics like Container<T>
                                types_to_define
                                    .push(format!("struct {}<T> {{ inner: T }}", type_name));
                            } else {
                                types_to_define.push(format!("struct {};", type_name));
                            }
                        }
                    }
                }
            }
        }

        // Prepend type definitions to the library
        if !types_to_define.is_empty() {
            let existing_content =
                fs::read_to_string(self.root.join("src/lib.rs")).unwrap_or_default();
            let type_defs = types_to_define.join("\n");
            let new_content = format!("{}\n\n{}", type_defs, existing_content);
            fs::write(self.root.join("src/lib.rs"), new_content).unwrap();
        }
    }
}
