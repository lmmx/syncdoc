use crate::TestCrate;
use std::fs;

pub fn auto_create_docs(test_crate: &TestCrate, code: &str) {
    // Always create the lib.md
    test_crate.write_doc("lib.md", "Test library");

    // Parse the code for items and create corresponding docs
    for line in code.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        // Match functions
        if let Some(fn_pos) = trimmed.find("fn ") {
            let before_fn = if fn_pos > 0 { &trimmed[..fn_pos] } else { "" };
            let is_valid_fn = fn_pos == 0 || before_fn.ends_with(' ') || before_fn.ends_with('\t');

            if is_valid_fn {
                let after_fn = &trimmed[fn_pos + 3..];
                if let Some(name_end) = after_fn.find(|c: char| c == '(' || c == '<') {
                    let clean_name = after_fn[..name_end].trim();
                    if !clean_name.is_empty() && !clean_name.contains(' ') {
                        eprintln!("Creating doc for function: {}", clean_name);
                        test_crate.write_doc(
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
                    test_crate.write_doc(
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
                    test_crate.write_doc(
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
                    test_crate.write_doc(
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
                    test_crate.write_doc(
                        &format!("lib/{}.md", clean_name),
                        &format!("Documentation for {}", clean_name),
                    );
                }
            }
        }
    }

    // Special handling for impl blocks and modules
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
                    if !field_name.is_empty()
                        && field_name.chars().all(|c| c.is_alphanumeric() || c == '_')
                    {
                        eprintln!("Creating doc for field: {}::{}", struct_name, field_name);
                        test_crate.write_doc(
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

        if (trimmed.starts_with("impl ") || trimmed.starts_with("impl<")) && !in_impl {
            in_impl = true;
            brace_depth = 0;

            let impl_part = trimmed.split('{').next().unwrap_or("");

            if let Some(for_pos) = impl_part.find(" for ") {
                // "impl Trait for Type"
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
                // "impl Type" or "impl<T> Type"
                let parts: Vec<&str> = impl_part.split_whitespace().collect();
                if let Some(last) = parts.last() {
                    impl_name = last.trim().split('<').next().unwrap_or(last).to_string();
                }
            }

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

        if in_impl {
            brace_depth += trimmed.matches('{').count();
            brace_depth -= trimmed.matches('}').count();

            // Look for function definitions
            if let Some(fn_pos) = trimmed.find("fn ") {
                let after_fn = &trimmed[fn_pos + 3..];
                if let Some(name_end) = after_fn.find(|c: char| c == '(' || c == '<') {
                    let clean_name = after_fn[..name_end].trim();
                    if !clean_name.is_empty() && !impl_name.is_empty() && !clean_name.contains(' ')
                    {
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

                        test_crate
                            .write_doc(&path_parts.join("/"), &format!("Documentation for method"));
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

                let dir_path = test_crate.root().join("docs").join(path_parts.join("/"));
                fs::create_dir_all(&dir_path).ok();

                eprintln!("Creating doc for module: {}", path_parts.join("::"));
                test_crate.write_doc(
                    &format!("{}.md", path_parts.join("/")),
                    &format!("Documentation for module"),
                );
            }
        }

        if in_mod {
            if trimmed.contains('{') {
                depth += trimmed.matches('{').count();
            }

            if line_no > mod_start_line {
                mod_content.push_str(line);
                mod_content.push('\n');
            }

            if trimmed.contains('}') {
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
}

fn create_docs_for_module_items(test_crate: &TestCrate, code: &str, module_path: &[String]) {
    for line in code.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        // Match functions
        if let Some(fn_pos) = trimmed.find("fn ") {
            let before_fn = if fn_pos > 0 { &trimmed[..fn_pos] } else { "" };
            let is_valid_fn = fn_pos == 0 || before_fn.ends_with(' ') || before_fn.ends_with('\t');

            if is_valid_fn {
                let after_fn = &trimmed[fn_pos + 3..];
                if let Some(name_end) = after_fn.find(|c: char| c == '(' || c == '<') {
                    let clean_name = after_fn[..name_end].trim();
                    if !clean_name.is_empty() && !clean_name.contains(' ') {
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
    }
}

fn handle_traits(test_crate: &TestCrate, code: &str) {
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
                    fs::create_dir_all(test_crate.root().join("docs/lib").join(&trait_name)).ok();
                    eprintln!("Creating doc for trait: {}", trait_name);
                    test_crate.write_doc(
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
