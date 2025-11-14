use super::regex::*;
use crate::TestCrate;
use regex::Regex;
use std::fs;

pub fn auto_create_docs(test_crate: &TestCrate, code: &str) {
    test_crate.write_doc("lib.md", "Test library");

    let items = [
        (&*FN_RE, "function"),
        (&*STRUCT_RE, "struct"),
        (&*ENUM_RE, "enum"),
        (&*CONST_RE, "const"),
        (&*TYPE_RE, "type alias"),
    ];

    for (regex, kind) in items {
        create_top_level_docs(test_crate, code, regex, kind);
    }

    handle_braced_blocks(
        test_crate,
        code,
        &STRUCT_RE,
        &FIELD_RE,
        false,
        |test_crate, parent, item| {
            eprintln!("Creating doc for field: {}::{}", parent, item);
            test_crate.write_doc(
                &format!("lib/{}/{}.md", parent, item),
                "Documentation for field",
            );
        },
    );

    handle_braced_blocks(
        test_crate,
        code,
        &TRAIT_RE,
        &FN_RE,
        true,
        |test_crate, parent, item| {
            eprintln!("Creating doc for trait method: {}::{}", parent, item);
            test_crate.write_doc(
                &format!("lib/{}/{}.md", parent, item),
                &format!("Documentation for {}::{}", parent, item),
            );
        },
    );

    handle_impl_and_modules(test_crate, code, Vec::new());
}

fn create_top_level_docs(test_crate: &TestCrate, code: &str, regex: &Regex, kind: &str) {
    for cap in regex.captures_iter(code) {
        if let Some(name) = cap.get(1).map(|m| m.as_str()).filter(|s| !s.is_empty()) {
            eprintln!("Creating doc for {}: {}", kind, name);
            test_crate.write_doc(
                &format!("lib/{}.md", name),
                &format!("Documentation for {}", name),
            );
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
    let mut i = 0;
    let lines: Vec<&str> = code.lines().collect();

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if let Some(cap) = MOD_RE.captures(trimmed) {
            if let Some(name) = cap.get(1).map(|m| m.as_str()) {
                let (end_idx, content) = extract_block_content(&lines, i);

                let mut path_parts = vec!["lib".to_string()];
                path_parts.extend(module_path.clone());
                path_parts.push(name.to_string());

                fs::create_dir_all(test_crate.root().join("docs").join(path_parts.join("/"))).ok();
                eprintln!("Creating doc for module: {}", path_parts.join("::"));
                test_crate.write_doc(
                    &format!("{}.md", path_parts.join("/")),
                    "Documentation for module",
                );

                let mut new_path = module_path.clone();
                new_path.push(name.to_string());

                // Process functions in module
                for line in content.lines() {
                    if let Some(fn_cap) = FN_RE.captures(line.trim()) {
                        if let Some(fn_name) =
                            fn_cap.get(1).map(|m| m.as_str()).filter(|s| !s.is_empty())
                        {
                            let mut fn_path_parts = vec!["lib".to_string()];
                            fn_path_parts.extend(new_path.clone());
                            fn_path_parts.push(format!("{}.md", fn_name));
                            eprintln!("Creating doc for function: {}", fn_name);
                            test_crate.write_doc(
                                &fn_path_parts.join("/"),
                                &format!("Documentation for {}", fn_name),
                            );
                        }
                    }
                }

                handle_impl_and_modules(test_crate, &content, new_path);
                i = end_idx;
                continue;
            }
        } else if let Some(cap) = IMPL_RE.captures(trimmed) {
            let impl_trait = cap.get(1).map(|m| m.as_str().to_string());
            if let Some(type_name) = cap.get(2).map(|m| m.as_str()).filter(|s| !s.is_empty()) {
                let (end_idx, content) = extract_block_content(&lines, i);

                let mut dir_path = test_crate.root().join("docs/lib");
                for module in &module_path {
                    dir_path = dir_path.join(module);
                }
                if let Some(ref trait_name) = impl_trait {
                    dir_path = dir_path.join(type_name).join(trait_name);
                } else {
                    dir_path = dir_path.join(type_name);
                }
                fs::create_dir_all(&dir_path).unwrap();

                // Process methods
                for line in content.lines() {
                    if let Some(fn_cap) = FN_RE.captures(line.trim()) {
                        if let Some(fn_name) =
                            fn_cap.get(1).map(|m| m.as_str()).filter(|s| !s.is_empty())
                        {
                            let mut path_parts = vec!["lib".to_string()];
                            path_parts.extend(module_path.clone());
                            if let Some(ref trait_name) = impl_trait {
                                eprintln!(
                                    "Creating doc for trait impl method: {}::{}::{}",
                                    type_name, trait_name, fn_name
                                );
                                path_parts
                                    .push(format!("{}/{}/{}.md", type_name, trait_name, fn_name));
                            } else {
                                eprintln!("Creating doc for method: {}::{}", type_name, fn_name);
                                path_parts.push(format!("{}/{}.md", type_name, fn_name));
                            }
                            test_crate.write_doc(&path_parts.join("/"), "Documentation for method");
                        }
                    }
                }

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
