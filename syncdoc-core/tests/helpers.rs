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
        // First, extract any existing type definitions from the code to avoid duplicates
        let existing_types = self.extract_existing_types(code);

        // Create dummy types, excluding ones that already exist - defined at crate root
        let dummy_types = self.create_dummy_types_str(code, &existing_types);

        // Inject the dummy types INTO the modules where they're used
        let code_with_types = self.inject_types_into_modules(code, &existing_types);

        let full_content = format!(
            r#"#![doc = include_str!("../docs/lib.md")]

    {}

    use syncdoc::omnidoc;

    #[omnidoc(path = "docs")]
    {}"#,
            dummy_types, code_with_types
        );
        fs::write(self.root.join("src/lib.rs"), full_content).unwrap();
    }

    fn extract_existing_types(&self, code: &str) -> std::collections::HashSet<String> {
        let mut types = std::collections::HashSet::new();

        for line in code.lines() {
            let trimmed = line.trim();

            // Extract struct names
            if let Some(struct_start) = trimmed
                .strip_prefix("struct ")
                .or(trimmed.strip_prefix("pub struct "))
            {
                if let Some(name) = struct_start
                    .split(|c: char| c.is_whitespace() || c == '{' || c == ';' || c == '<')
                    .next()
                {
                    let clean_name = name.trim().to_string();
                    types.insert(clean_name.clone());

                    // Also check if it's a generic definition
                    if struct_start.contains('<') && struct_start.contains('>') {
                        types.insert(format!("{}<T>", clean_name));
                    }
                }
            }

            // Extract trait names
            if let Some(trait_start) = trimmed
                .strip_prefix("trait ")
                .or(trimmed.strip_prefix("pub trait "))
            {
                if let Some(name) = trait_start
                    .split(|c: char| c.is_whitespace() || c == '{' || c == '<')
                    .next()
                {
                    types.insert(name.trim().to_string());
                }
            }

            // Extract enum names
            if let Some(enum_start) = trimmed
                .strip_prefix("enum ")
                .or(trimmed.strip_prefix("pub enum "))
            {
                if let Some(name) = enum_start
                    .split(|c: char| c.is_whitespace() || c == '{' || c == '<')
                    .next()
                {
                    types.insert(name.trim().to_string());
                }
            }
        }

        types
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

                        // Now look for fields in subsequent lines
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

            // Match: type Alias = ...;
            if trimmed.starts_with("type ") || trimmed.starts_with("pub type ") {
                let type_start = trimmed
                    .strip_prefix("pub type ")
                    .or_else(|| trimmed.strip_prefix("type "))
                    .unwrap();

                if let Some(name) = type_start
                    .split(|c: char| c.is_whitespace() || c == '=' || c == '<')
                    .next()
                {
                    let clean_name = name.trim();
                    if !clean_name.is_empty() {
                        eprintln!("Creating doc for type alias: {}", clean_name);
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
        self.handle_traits(code);
        self.parse_struct_fields(code);
    }

    fn parse_struct_fields(&self, code: &str) {
        let mut in_struct = false;
        let mut struct_name = String::new();
        let mut brace_depth = 0;

        for line in code.lines() {
            let trimmed = line.trim();

            // Detect struct start
            if !in_struct
                && (trimmed.starts_with("struct ") || trimmed.starts_with("pub struct "))
                && trimmed.contains('{')
            {
                let struct_start = trimmed
                    .strip_prefix("pub struct ")
                    .or_else(|| trimmed.strip_prefix("struct "))
                    .unwrap();

                if let Some(name) = struct_start
                    .split(|c: char| c.is_whitespace() || c == '{' || c == '<')
                    .next()
                {
                    struct_name = name.trim().to_string();
                    in_struct = true;
                    brace_depth = 0;
                }
            }

            if in_struct {
                brace_depth += trimmed.matches('{').count();
                brace_depth -= trimmed.matches('}').count();

                // Look for field: Type pattern
                if trimmed.contains(':') && !trimmed.starts_with("//") {
                    let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let field_name = parts[0].trim().trim_start_matches("pub").trim();
                        // Make sure it's a valid identifier
                        if !field_name.is_empty()
                            && field_name.chars().all(|c| c.is_alphanumeric() || c == '_')
                        {
                            eprintln!("Creating doc for field: {}::{}", struct_name, field_name);
                            self.write_doc(
                                &format!("lib/{}/{}.md", struct_name, field_name),
                                &format!("Documentation for field"),
                            );
                        }
                    }
                }

                if brace_depth == 0 {
                    in_struct = false;
                    struct_name.clear();
                }
            }
        }
    }

    fn handle_impl_blocks(&self, code: &str) {
        self.handle_impl_blocks_with_context(code, Vec::new());
    }

    fn handle_impl_blocks_with_context(&self, code: &str, module_path: Vec<String>) {
        let mut in_impl = false;
        let mut impl_name = String::new();
        let mut impl_trait = Option::<String>::None;
        let mut brace_depth = 0;

        for line in code.lines() {
            let trimmed = line.trim();

            if (trimmed.starts_with("impl ") || trimmed.starts_with("impl<")) && !in_impl {
                in_impl = true;
                brace_depth = 0;

                // Extract type name (handle both "impl Type" and "impl Trait for Type")
                let impl_part = trimmed.split('{').next().unwrap_or("");

                if let Some(for_pos) = impl_part.find(" for ") {
                    // "impl Trait for Type" - use Type, and note Trait
                    let trait_part = &impl_part[4..for_pos].trim();
                    let trait_name = trait_part
                        .split_whitespace()
                        .filter(|s| *s != "unsafe" && !s.starts_with('<'))
                        .last()
                        .unwrap_or("")
                        .split('<')
                        .next()
                        .unwrap_or("")
                        .trim();
                    impl_trait = Some(trait_name.to_string());

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
                    let mut dir_path = self.root.join("docs/lib");
                    for module in &module_path {
                        dir_path = dir_path.join(module);
                    }

                    if let Some(ref trait_name) = impl_trait {
                        // Create Type/Trait directory structure
                        dir_path = dir_path.join(&impl_name).join(trait_name);
                    } else {
                        // Create just Type directory
                        dir_path = dir_path.join(&impl_name);
                    }

                    fs::create_dir_all(&dir_path).ok();
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
                            let mut path_parts = vec!["lib".to_string()];
                            path_parts.extend(module_path.clone());

                            if let Some(ref trait_name) = impl_trait {
                                eprintln!(
                                    "Creating doc for trait impl method: {}::{}::{}",
                                    impl_name, trait_name, clean_name
                                );
                                path_parts.push(format!(
                                    "{}/{}/{}.md",
                                    impl_name, trait_name, clean_name
                                ));
                            } else {
                                eprintln!("Creating doc for method: {}::{}", impl_name, clean_name);
                                path_parts.push(format!("{}/{}.md", impl_name, clean_name));
                            }

                            self.write_doc(
                                &path_parts.join("/"),
                                &format!("Documentation for method"),
                            );
                        }
                    }
                }

                // Exit impl block when braces are balanced
                if brace_depth == 0 && trimmed.ends_with('}') {
                    in_impl = false;
                    impl_name.clear();
                    impl_trait = None;
                }
            }
        }
    }

    fn handle_modules(&self, code: &str) {
        self.handle_modules_recursive(code, Vec::new());
    }

    fn handle_modules_recursive(&self, code: &str, parent_modules: Vec<String>) {
        let mut in_mod = false;
        let mut mod_name = String::new();
        let mut depth = 0;
        let mut mod_content = String::new();
        let mut mod_start_line = 0;

        for (line_no, line) in code.lines().enumerate() {
            let trimmed = line.trim();

            if !in_mod && (trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ")) {
                let mod_start = trimmed
                    .strip_prefix("pub mod ")
                    .or_else(|| trimmed.strip_prefix("mod "))
                    .unwrap();

                if let Some(name) = mod_start.split('{').next() {
                    mod_name = name.trim().to_string();
                    in_mod = true;
                    depth = 0;
                    mod_start_line = line_no;
                    mod_content.clear();

                    // Create module directory
                    let mut path_parts = vec!["lib".to_string()];
                    path_parts.extend(parent_modules.clone());
                    path_parts.push(mod_name.clone());

                    let dir_path = self.root.join("docs").join(path_parts.join("/"));
                    fs::create_dir_all(&dir_path).ok();

                    eprintln!("Creating doc for module: {}", path_parts.join("::"));
                    self.write_doc(
                        &format!("{}.md", path_parts.join("/")),
                        &format!("Documentation for module"),
                    );
                }
            }

            if in_mod {
                if trimmed.contains('{') {
                    depth += trimmed.matches('{').count();
                }

                // Collect module content
                if line_no > mod_start_line {
                    mod_content.push_str(line);
                    mod_content.push('\n');
                }

                if trimmed.contains('}') {
                    depth -= trimmed.matches('}').count();
                    if depth == 0 {
                        // Process the module content
                        let mut new_path = parent_modules.clone();
                        new_path.push(mod_name.clone());

                        // Create docs for functions inside this module
                        self.create_docs_for_module_items(&mod_content, &new_path);

                        // Recursively handle nested modules
                        self.handle_modules_recursive(&mod_content, new_path.clone());

                        // Handle impl blocks in this module
                        self.handle_impl_blocks_with_context(&mod_content, new_path);

                        in_mod = false;
                        mod_name.clear();
                        mod_content.clear();
                    }
                }
            }
        }
    }

    fn create_docs_for_module_items(&self, code: &str, module_path: &[String]) {
        for line in code.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            // Match functions
            if let Some(fn_pos) = trimmed.find("fn ") {
                let before_fn = if fn_pos > 0 { &trimmed[..fn_pos] } else { "" };
                let is_valid_fn =
                    fn_pos == 0 || before_fn.ends_with(' ') || before_fn.ends_with('\t');

                if is_valid_fn {
                    let after_fn = &trimmed[fn_pos + 3..];
                    if let Some(name_end) = after_fn.find(|c: char| c == '(' || c == '<') {
                        let clean_name = after_fn[..name_end].trim();
                        if !clean_name.is_empty() && !clean_name.contains(' ') {
                            let mut path_parts = vec!["lib".to_string()];
                            path_parts.extend(module_path.iter().cloned());
                            path_parts.push(format!("{}.md", clean_name));

                            eprintln!("Creating doc for function: {}", clean_name);
                            self.write_doc(
                                &path_parts.join("/"),
                                &format!("Documentation for {}", clean_name),
                            );
                        }
                    }
                }
            }

            // Could add struct/enum/const detection here too if needed
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
    fn create_dummy_types_str(
        &self,
        code: &str,
        existing_types: &std::collections::HashSet<String>,
    ) -> String {
        let mut types_to_define = Vec::new();
        let mut trait_methods: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for line in code.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("impl ") || trimmed.starts_with("impl<") {
                let impl_part = trimmed.split('{').next().unwrap_or("");

                if let Some(for_pos) = impl_part.find(" for ") {
                    // "impl Trait for Type"
                    let trait_part = impl_part[4..for_pos].trim();
                    let type_part = impl_part[for_pos + 5..].trim();

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

                    if !trait_name.is_empty()
                        && trait_name.chars().next().unwrap().is_uppercase()
                        && !existing_types.contains(trait_name)
                    {
                        // Initialize trait for method collection
                        trait_methods
                            .entry(trait_name.to_string())
                            .or_insert_with(Vec::new);
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

                    if !type_name.is_empty()
                        && type_name.chars().next().unwrap().is_uppercase()
                        && !existing_types.contains(type_name)
                    {
                        types_to_define.push(format!("pub struct {};", type_name));
                    }
                } else {
                    // "impl Type" or "impl<T> Type"
                    let parts: Vec<&str> = impl_part.split_whitespace().collect();
                    if let Some(last) = parts.last() {
                        let type_name = last.split('<').next().unwrap_or(last).trim();

                        if !type_name.is_empty()
                            && type_name
                                .chars()
                                .next()
                                .map(|c| c.is_uppercase())
                                .unwrap_or(false)
                            && !existing_types.contains(type_name)
                        {
                            // Check if impl block has generics like impl<T> GenericStruct<T>
                            let has_generics = last.contains('<') && last.contains('>');

                            let def = if has_generics {
                                format!(
                                    "pub struct {}<T> {{ _inner: std::marker::PhantomData<T> }}",
                                    type_name
                                )
                            } else {
                                format!("pub struct {};", type_name)
                            };

                            types_to_define.push(def);
                        }
                    }
                }
            }
        }

        // Now scan for methods inside trait impls to add to trait definitions
        let mut in_trait_impl = false;
        let mut current_trait = String::new();
        let mut depth = 0;

        for line in code.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("impl ") && trimmed.contains(" for ") {
                if let Some(impl_part) = trimmed.split('{').next() {
                    if let Some(for_pos) = impl_part.find(" for ") {
                        let trait_part = impl_part[4..for_pos].trim();
                        let trait_name = trait_part
                            .split_whitespace()
                            .last()
                            .unwrap_or("")
                            .split('<')
                            .next()
                            .unwrap_or("")
                            .trim();

                        if !trait_name.is_empty() {
                            current_trait = trait_name.to_string();
                            in_trait_impl = true;
                            depth = 0;
                        }
                    }
                }
            }

            if in_trait_impl {
                depth += trimmed.matches('{').count();
                depth -= trimmed.matches('}').count();

                // Extract method signatures
                if trimmed.contains("fn ") && !current_trait.is_empty() {
                    // Simple method signature extraction
                    if let Some(fn_pos) = trimmed.find("fn ") {
                        let after_fn = &trimmed[fn_pos..];
                        if let Some(body_start) = after_fn.find('{') {
                            let sig = after_fn[..body_start].trim().to_string() + ";";
                            if let Some(methods) = trait_methods.get_mut(&current_trait) {
                                methods.push(sig);
                            }
                        }
                    }
                }

                if depth == 0 {
                    in_trait_impl = false;
                    current_trait.clear();
                }
            }
        }

        // Generate trait definitions with methods
        for (trait_name, methods) in trait_methods {
            if methods.is_empty() {
                types_to_define.push(format!("pub trait {} {{}}", trait_name));
            } else {
                let methods_str = methods.join("\n    ");
                types_to_define.push(format!(
                    "pub trait {} {{\n    {}\n}}",
                    trait_name, methods_str
                ));
            }
        }

        types_to_define.join("\n")
    }

    fn handle_traits(&self, code: &str) {
        let mut in_trait = false;
        let mut trait_name = String::new();
        let mut brace_depth = 0;

        for line in code.lines() {
            let trimmed = line.trim();

            if (trimmed.starts_with("trait ") || trimmed.starts_with("pub trait ")) && !in_trait {
                in_trait = true;
                brace_depth = 0;

                let trait_start = trimmed
                    .strip_prefix("pub trait ")
                    .or_else(|| trimmed.strip_prefix("trait "))
                    .unwrap();

                if let Some(name) = trait_start
                    .split(|c: char| c.is_whitespace() || c == '{' || c == '<')
                    .next()
                {
                    trait_name = name.trim().to_string();
                    if !trait_name.is_empty() {
                        fs::create_dir_all(self.root.join("docs/lib").join(&trait_name)).ok();
                        eprintln!("Creating doc for trait: {}", trait_name);
                        self.write_doc(
                            &format!("lib/{}.md", trait_name),
                            &format!("Documentation for {}", trait_name),
                        );
                    }
                }
            }

            if in_trait {
                brace_depth += trimmed.matches('{').count();
                brace_depth -= trimmed.matches('}').count();

                // Look for methods with bodies (default implementations)
                if let Some(fn_pos) = trimmed.find("fn ") {
                    let after_fn = &trimmed[fn_pos + 3..];
                    if let Some(name_end) = after_fn.find(|c: char| c == '(' || c == '<') {
                        let clean_name = after_fn[..name_end].trim();
                        // Check if this method has a body (contains '{' after the signature)
                        let rest_of_line = &trimmed[fn_pos..];
                        if !clean_name.is_empty()
                            && !trait_name.is_empty()
                            && !clean_name.contains(' ')
                            && rest_of_line.contains('{')
                        {
                            eprintln!(
                                "Creating doc for trait method: {}::{}",
                                trait_name, clean_name
                            );
                            self.write_doc(
                                &format!("lib/{}/{}.md", trait_name, clean_name),
                                &format!("Documentation for {}::{}", trait_name, clean_name),
                            );
                        }
                    }
                }

                if brace_depth == 0 && trimmed.ends_with('}') {
                    in_trait = false;
                    trait_name.clear();
                }
            }
        }
    }

    fn inject_types_into_modules(
        &self,
        code: &str,
        existing_types: &std::collections::HashSet<String>,
    ) -> String {
        let mut result = String::new();
        let mut in_module = false;
        let mut module_depth = 0;
        let mut types_needed_in_module = Vec::new();
        let mut module_content = String::new();

        for line in code.lines() {
            let trimmed = line.trim();

            // Entering a module
            if (trimmed.starts_with("mod ") || trimmed.starts_with("pub mod "))
                && trimmed.contains('{')
            {
                in_module = true;
                module_depth = 0;
                types_needed_in_module.clear();
                module_content.clear();
                result.push_str(line);
                result.push('\n');
                continue;
            }

            if in_module {
                module_depth += trimmed.matches('{').count();
                module_depth -= trimmed.matches('}').count();

                // Collect what types are referenced in impl blocks within this module
                if trimmed.starts_with("impl ") || trimmed.starts_with("impl<") {
                    let impl_part = trimmed.split('{').next().unwrap_or("");

                    if let Some(for_pos) = impl_part.find(" for ") {
                        let type_part = impl_part[for_pos + 5..].trim();
                        let type_name = type_part
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .split('<')
                            .next()
                            .unwrap_or("")
                            .trim();
                        if !type_name.is_empty() && !existing_types.contains(type_name) {
                            if !types_needed_in_module.contains(&type_name.to_string()) {
                                types_needed_in_module.push(type_name.to_string());
                            }
                        }
                    } else {
                        let parts: Vec<&str> = impl_part.split_whitespace().collect();
                        if let Some(last) = parts.last() {
                            let type_name = last.split('<').next().unwrap_or(last).trim();
                            if !type_name.is_empty() && !existing_types.contains(type_name) {
                                if !types_needed_in_module.contains(&type_name.to_string()) {
                                    types_needed_in_module.push(type_name.to_string());
                                }
                            }
                        }
                    }
                }

                module_content.push_str(line);
                module_content.push('\n');

                // Exiting module
                if module_depth == 0 && trimmed.ends_with('}') {
                    // Inject type imports at the beginning of the module
                    if !types_needed_in_module.is_empty() {
                        let mut injected_module = String::new();
                        for type_name in &types_needed_in_module {
                            injected_module.push_str(&format!("    use crate::{};\n", type_name));
                        }
                        injected_module.push_str("\n");
                        injected_module.push_str(&module_content);

                        result.push_str(&injected_module);
                    } else {
                        result.push_str(&module_content);
                    }

                    in_module = false;
                    module_content.clear();
                    types_needed_in_module.clear();
                    continue;
                }
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }

        result
    }
}
