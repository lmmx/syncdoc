use super::regex::*;
use crate::TestCrate;
use regex::Regex;
use std::fs;

pub fn auto_create_docs(test_crate: &TestCrate, code: &str) {
    test_crate.write_doc("lib.md", "Test library");

    // Create top-level docs
    for (regex, _kind) in [
        (&*FN_RE, "function"),
        (&*STRUCT_RE, "struct"),
        (&*ENUM_RE, "enum"),
        (&*CONST_RE, "const"),
        (&*TYPE_RE, "type alias"),
        (&*TRAIT_RE, "trait"),
    ] {
        for cap in regex.captures_iter(code) {
            if let Some(name) = cap.get(1).map(|m| m.as_str()).filter(|s| !s.is_empty()) {
                test_crate.write_doc(
                    &format!("lib/{}.md", name),
                    &format!("Documentation for {}", name),
                );
            }
        }
    }

    // Handle struct fields
    handle_braced_blocks(
        test_crate,
        code,
        &STRUCT_RE,
        &FIELD_RE,
        false,
        |tc, parent, item| {
            tc.write_doc(
                &format!("lib/{}/{}.md", parent, item),
                "Documentation for field",
            );
        },
    );

    // Handle trait methods - special case to handle both declarations and implementations
    handle_trait_methods(test_crate, code);

    handle_impl_and_modules(test_crate, code, Vec::new());
}

fn handle_trait_methods(test_crate: &TestCrate, code: &str) {
    let mut in_trait = false;
    let mut trait_name = String::new();
    let mut brace_depth = 0;

    for line in code.lines() {
        let trimmed = line.trim();

        if !in_trait && TRAIT_RE.is_match(trimmed) && trimmed.contains('{') {
            if let Some(name) = TRAIT_RE
                .captures(trimmed)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str())
            {
                trait_name = name.to_string();
                in_trait = true;
                brace_depth = 0;
            }
        }

        if in_trait {
            brace_depth += trimmed.matches('{').count();
            brace_depth -= trimmed.matches('}').count();

            if !trimmed.starts_with("//") {
                if let Some(method_name) = FN_RE
                    .captures(trimmed)
                    .and_then(|cap| cap.get(1))
                    .map(|m| m.as_str())
                    .filter(|s| !s.is_empty())
                {
                    // Accept both declarations (with ;) and implementations (with {)
                    if trimmed.contains('{') || trimmed.contains(';') {
                        test_crate.write_doc(
                            &format!("lib/{}/{}.md", trait_name, method_name),
                            &format!("Documentation for {}::{}", trait_name, method_name),
                        );
                    }
                }
            }

            if brace_depth == 0 {
                in_trait = false;
                trait_name.clear();
            }
        }
    }
}

fn handle_braced_blocks<F>(
    test_crate: &TestCrate,
    code: &str,
    start_regex: &Regex,
    item_regex: &Regex,
    require_body: bool,
    mut handler: F,
) where
    F: FnMut(&TestCrate, &str, &str),
{
    let mut in_block = false;
    let mut parent_name = String::new();
    let mut brace_depth = 0;

    for line in code.lines() {
        let trimmed = line.trim();

        if !in_block && start_regex.is_match(trimmed) && trimmed.contains('{') {
            if let Some(name) = start_regex
                .captures(trimmed)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str())
            {
                parent_name = name.to_string();
                in_block = true;
                brace_depth = 0;
            }
        }

        if in_block {
            brace_depth += trimmed.matches('{').count();
            brace_depth -= trimmed.matches('}').count();

            if !trimmed.starts_with("//") {
                if let Some(item) = item_regex
                    .captures(trimmed)
                    .and_then(|cap| cap.get(1))
                    .map(|m| m.as_str())
                    .filter(|s| !s.is_empty() && (!require_body || trimmed.contains('{')))
                {
                    handler(test_crate, &parent_name, item);
                }
            }

            if brace_depth == 0 {
                in_block = false;
                parent_name.clear();
            }
        }
    }
}

fn handle_impl_and_modules(test_crate: &TestCrate, code: &str, module_path: Vec<String>) {
    let lines: Vec<&str> = code.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if let Some(cap) = MOD_RE.captures(trimmed) {
            if let Some(name) = cap.get(1).map(|m| m.as_str()) {
                let (end_idx, content) = extract_block_content(&lines, i);
                let mut new_path = module_path.clone();
                new_path.push(name.to_string());

                let path = format!("lib/{}.md", new_path.join("/"));
                test_crate.write_doc(&path, "Documentation for module");

                // Document functions in module
                FN_RE
                    .captures_iter(&content)
                    .filter_map(|c| c.get(1).map(|m| m.as_str()).filter(|s| !s.is_empty()))
                    .for_each(|fn_name| {
                        test_crate.write_doc(
                            &format!("lib/{}/{}.md", new_path.join("/"), fn_name),
                            &format!("Documentation for {}", fn_name),
                        );
                    });

                handle_impl_and_modules(test_crate, &content, new_path);
                i = end_idx;
                continue;
            }
        } else if let Some(cap) = IMPL_RE.captures(trimmed) {
            let impl_trait = cap.get(1).map(|m| m.as_str());
            if let Some(type_name) = cap.get(2).map(|m| m.as_str()).filter(|s| !s.is_empty()) {
                let (end_idx, content) = extract_block_content(&lines, i);

                let mut dir_path = test_crate.root().join("docs/lib");
                for module in &module_path {
                    dir_path = dir_path.join(module);
                }
                dir_path = if let Some(trait_name) = impl_trait {
                    dir_path.join(type_name).join(trait_name)
                } else {
                    dir_path.join(type_name)
                };
                fs::create_dir_all(&dir_path).unwrap();

                // Document methods
                FN_RE
                    .captures_iter(&content)
                    .filter_map(|c| c.get(1).map(|m| m.as_str()).filter(|s| !s.is_empty()))
                    .for_each(|fn_name| {
                        let mut path_parts = vec!["lib".to_string()];
                        path_parts.extend(module_path.clone());
                        if let Some(trait_name) = impl_trait {
                            path_parts.push(format!("{}/{}/{}.md", type_name, trait_name, fn_name));
                        } else {
                            path_parts.push(format!("{}/{}.md", type_name, fn_name));
                        }
                        test_crate.write_doc(&path_parts.join("/"), "Documentation for method");
                    });

                i = end_idx;
                continue;
            }
        }
        i += 1;
    }
}

fn extract_block_content(lines: &[&str], start_idx: usize) -> (usize, String) {
    let mut depth = 1;
    let mut content = String::new();
    let mut i = start_idx + 1;

    while i < lines.len() && depth > 0 {
        depth += lines[i].matches('{').count();
        depth -= lines[i].matches('}').count();
        if depth > 0 {
            content.push_str(lines[i]);
            content.push('\n');
        }
        i += 1;
    }

    (i, content)
}
