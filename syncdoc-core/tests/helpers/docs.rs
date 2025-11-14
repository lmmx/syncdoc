use crate::TestCrate;
use regex::Regex;
use std::fs;
use std::sync::LazyLock;

static FN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bfn\s+(\w+)\s*[<(]").unwrap());

static STRUCT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:pub\s+)?struct\s+(\w+)").unwrap());

static ENUM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:pub\s+)?enum\s+(\w+)").unwrap());

static CONST_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:pub\s+)?const\s+(\w+)\s*:").unwrap());

static TYPE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:pub\s+)?type\s+(\w+)").unwrap());

static IMPL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"impl(?:<[^>]+>)?\s+(?:(\w+)\s+for\s+)?(\w+)").unwrap());

static TRAIT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:pub\s+)?trait\s+(\w+)").unwrap());

static MOD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:pub\s+)?mod\s+(\w+)\s*\{").unwrap());

static FIELD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:pub\s+)?(\w+)\s*:").unwrap());

pub fn auto_create_docs(test_crate: &TestCrate, code: &str) {
    test_crate.write_doc("lib.md", "Test library");

    // Functions
    for cap in FN_RE.captures_iter(code) {
        if let Some(name) = cap.get(1) {
            let clean_name = name.as_str();
            if !clean_name.is_empty() {
                eprintln!("Creating doc for function: {}", clean_name);
                test_crate.write_doc(
                    &format!("lib/{}.md", clean_name),
                    &format!("Documentation for {}", clean_name),
                );
            }
        }
    }

    // Structs
    for cap in STRUCT_RE.captures_iter(code) {
        if let Some(name) = cap.get(1) {
            let clean_name = name.as_str();
            eprintln!("Creating doc for struct: {}", clean_name);
            test_crate.write_doc(
                &format!("lib/{}.md", clean_name),
                &format!("Documentation for {}", clean_name),
            );
        }
    }

    // Enums
    for cap in ENUM_RE.captures_iter(code) {
        if let Some(name) = cap.get(1) {
            let clean_name = name.as_str();
            eprintln!("Creating doc for enum: {}", clean_name);
            test_crate.write_doc(
                &format!("lib/{}.md", clean_name),
                &format!("Documentation for {}", clean_name),
            );
        }
    }

    // Consts
    for cap in CONST_RE.captures_iter(code) {
        if let Some(name) = cap.get(1) {
            let clean_name = name.as_str();
            eprintln!("Creating doc for const: {}", clean_name);
            test_crate.write_doc(
                &format!("lib/{}.md", clean_name),
                &format!("Documentation for {}", clean_name),
            );
        }
    }

    // Type aliases
    for cap in TYPE_RE.captures_iter(code) {
        if let Some(name) = cap.get(1) {
            let clean_name = name.as_str();
            eprintln!("Creating doc for type alias: {}", clean_name);
            test_crate.write_doc(
                &format!("lib/{}.md", clean_name),
                &format!("Documentation for {}", clean_name),
            );
        }
    }

    handle_impl_blocks(test_crate, code);
    handle_modules(test_crate, code);
    handle_traits(test_crate, code);
    parse_struct_fields(test_crate, code);
}

fn parse_struct_fields(test_crate: &TestCrate, code: &str) {
    let mut in_struct = false;
    let mut struct_name = String::new();
    let mut brace_depth = 0;

    for line in code.lines() {
        let trimmed = line.trim();

        if !in_struct {
            if let Some(cap) = STRUCT_RE.captures(trimmed) {
                if trimmed.contains('{') {
                    if let Some(name) = cap.get(1) {
                        struct_name = name.as_str().to_string();
                        in_struct = true;
                        brace_depth = 0;
                    }
                }
            }
        }

        if in_struct {
            brace_depth += trimmed.matches('{').count();
            brace_depth -= trimmed.matches('}').count();

            if let Some(cap) = FIELD_RE.captures(trimmed) {
                if !trimmed.starts_with("//") {
                    if let Some(field) = cap.get(1) {
                        let field_name = field.as_str();
                        eprintln!("Creating doc for field: {}::{}", struct_name, field_name);
                        test_crate.write_doc(
                            &format!("lib/{}/{}.md", struct_name, field_name),
                            "Documentation for field",
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

fn handle_impl_blocks(test_crate: &TestCrate, code: &str) {
    handle_impl_blocks_with_context(test_crate, code, Vec::new());
}

fn handle_impl_blocks_with_context(test_crate: &TestCrate, code: &str, module_path: Vec<String>) {
    let mut in_impl = false;
    let mut impl_name = String::new();
    let mut impl_trait = Option::<String>::None;
    let mut brace_depth = 0;

    for line in code.lines() {
        let trimmed = line.trim();

        if !in_impl {
            if let Some(cap) = IMPL_RE.captures(trimmed) {
                in_impl = true;
                brace_depth = 0;

                // cap.get(1) is trait name (in "impl Trait for Type")
                // cap.get(2) is type name
                impl_trait = cap.get(1).map(|m| m.as_str().to_string());
                impl_name = cap.get(2).map(|m| m.as_str()).unwrap_or("").to_string();

                if !impl_name.is_empty() {
                    let mut dir_path = test_crate.root().join("docs/lib");
                    for module in &module_path {
                        dir_path = dir_path.join(module);
                    }

                    if let Some(ref trait_name) = impl_trait {
                        dir_path = dir_path.join(&impl_name).join(trait_name);
                    } else {
                        dir_path = dir_path.join(&impl_name);
                    }

                    fs::create_dir_all(&dir_path).ok();
                }
            }
        }

        if in_impl {
            brace_depth += trimmed.matches('{').count();
            brace_depth -= trimmed.matches('}').count();

            // Look for function definitions
            if let Some(cap) = FN_RE.captures(trimmed) {
                if let Some(name) = cap.get(1) {
                    let clean_name = name.as_str();
                    if !clean_name.is_empty() && !impl_name.is_empty() {
                        let mut path_parts = vec!["lib".to_string()];
                        path_parts.extend(module_path.clone());

                        if let Some(ref trait_name) = impl_trait {
                            eprintln!(
                                "Creating doc for trait impl method: {}::{}::{}",
                                impl_name, trait_name, clean_name
                            );
                            path_parts
                                .push(format!("{}/{}/{}.md", impl_name, trait_name, clean_name));
                        } else {
                            eprintln!("Creating doc for method: {}::{}", impl_name, clean_name);
                            path_parts.push(format!("{}/{}.md", impl_name, clean_name));
                        }

                        test_crate.write_doc(&path_parts.join("/"), "Documentation for method");
                    }
                }
            }

            if brace_depth == 0 && trimmed.ends_with('}') {
                in_impl = false;
                impl_name.clear();
                impl_trait = None;
            }
        }
    }
}

fn handle_modules(test_crate: &TestCrate, code: &str) {
    handle_modules_recursive(test_crate, code, Vec::new());
}

fn handle_modules_recursive(test_crate: &TestCrate, code: &str, parent_modules: Vec<String>) {
    let mut in_mod = false;
    let mut mod_name = String::new();
    let mut depth = 0;
    let mut mod_content = String::new();
    let mut mod_start_line = 0;

    for (line_no, line) in code.lines().enumerate() {
        let trimmed = line.trim();

        if !in_mod {
            if let Some(cap) = MOD_RE.captures(trimmed) {
                if let Some(name) = cap.get(1) {
                    mod_name = name.as_str().to_string();
                    in_mod = true;
                    depth = 0;
                    mod_start_line = line_no;
                    mod_content.clear();

                    // Create module directory
                    let mut path_parts = vec!["lib".to_string()];
                    path_parts.extend(parent_modules.clone());
                    path_parts.push(mod_name.clone());

                    let dir_path = test_crate.root().join("docs").join(path_parts.join("/"));
                    fs::create_dir_all(&dir_path).ok();

                    eprintln!("Creating doc for module: {}", path_parts.join("::"));
                    test_crate.write_doc(
                        &format!("{}.md", path_parts.join("/")),
                        "Documentation for module",
                    );
                }
            }
        }

        if in_mod {
            depth += trimmed.matches('{').count();

            if line_no > mod_start_line {
                mod_content.push_str(line);
                mod_content.push('\n');
            }

            depth -= trimmed.matches('}').count();

            if depth == 0 {
                let mut new_path = parent_modules.clone();
                new_path.push(mod_name.clone());

                create_docs_for_module_items(test_crate, &mod_content, &new_path);
                handle_modules_recursive(test_crate, &mod_content, new_path.clone());
                handle_impl_blocks_with_context(test_crate, &mod_content, new_path);

                in_mod = false;
                mod_name.clear();
                mod_content.clear();
            }
        }
    }
}

fn create_docs_for_module_items(test_crate: &TestCrate, code: &str, module_path: &[String]) {
    // Functions in modules
    for cap in FN_RE.captures_iter(code) {
        if let Some(name) = cap.get(1) {
            let clean_name = name.as_str();
            if !clean_name.is_empty() {
                let mut path_parts = vec!["lib".to_string()];
                path_parts.extend(module_path.iter().cloned());
                path_parts.push(format!("{}.md", clean_name));

                eprintln!("Creating doc for function: {}", clean_name);
                test_crate.write_doc(
                    &path_parts.join("/"),
                    &format!("Documentation for {}", clean_name),
                );
            }
        }
    }
}

fn handle_traits(test_crate: &TestCrate, code: &str) {
    let mut in_trait = false;
    let mut trait_name = String::new();
    let mut brace_depth = 0;

    for line in code.lines() {
        let trimmed = line.trim();

        if !in_trait {
            if let Some(cap) = TRAIT_RE.captures(trimmed) {
                in_trait = true;
                brace_depth = 0;

                if let Some(name) = cap.get(1) {
                    trait_name = name.as_str().to_string();
                    if !trait_name.is_empty() {
                        fs::create_dir_all(test_crate.root().join("docs/lib").join(&trait_name))
                            .ok();
                        eprintln!("Creating doc for trait: {}", trait_name);
                        test_crate.write_doc(
                            &format!("lib/{}.md", trait_name),
                            &format!("Documentation for {}", trait_name),
                        );
                    }
                }
            }
        }

        if in_trait {
            brace_depth += trimmed.matches('{').count();
            brace_depth -= trimmed.matches('}').count();

            // Look for methods with bodies (default implementations)
            if let Some(cap) = FN_RE.captures(trimmed) {
                if let Some(name) = cap.get(1) {
                    let clean_name = name.as_str();
                    // Check if this function has a body (default implementation)
                    if !clean_name.is_empty() && !trait_name.is_empty() && trimmed.contains('{') {
                        eprintln!(
                            "Creating doc for trait method: {}::{}",
                            trait_name, clean_name
                        );
                        test_crate.write_doc(
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
