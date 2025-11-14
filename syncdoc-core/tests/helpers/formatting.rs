// syncdoc-core/tests/helpers/formatting.rs

use super::parser::{parse_rust_code, ItemKind};
use std::collections::{HashMap, HashSet};
use std::process::Command;

pub fn create_dummy_types_str(code: &str, existing_types: &HashSet<String>) -> String {
    let items = parse_rust_code(code);
    let mut types_to_define = Vec::new();
    let mut trait_methods: HashMap<String, Vec<String>> = HashMap::new();

    for item in items {
        match item.kind {
            ItemKind::ImplBlock {
                trait_name: Some(trait_name),
                type_name,
            } => {
                if !existing_types.contains(&trait_name) {
                    trait_methods.entry(trait_name).or_insert_with(Vec::new);
                }
                if !existing_types.contains(&type_name)
                    && type_name.chars().next().unwrap().is_uppercase()
                {
                    let def = if item.has_generics {
                        format!(
                            "pub struct {}<T> {{ _inner: std::marker::PhantomData<T> }}",
                            type_name
                        )
                    } else {
                        format!("pub struct {};", type_name)
                    };
                    if !types_to_define.contains(&def) {
                        types_to_define.push(def);
                    }
                }
            }
            ItemKind::ImplBlock {
                trait_name: None,
                type_name,
            } => {
                if !existing_types.contains(&type_name)
                    && type_name.chars().next().unwrap().is_uppercase()
                {
                    let def = if item.has_generics {
                        format!(
                            "pub struct {}<T> {{ _inner: std::marker::PhantomData<T> }}",
                            type_name
                        )
                    } else {
                        format!("pub struct {};", type_name)
                    };
                    if !types_to_define.contains(&def) {
                        types_to_define.push(def);
                    }
                }
            }
            _ => {}
        }
    }

    extract_trait_methods(code, &mut trait_methods);

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

fn extract_trait_methods(code: &str, trait_methods: &mut HashMap<String, Vec<String>>) {
    let mut in_trait_impl = false;
    let mut current_trait = String::new();
    let mut depth = 0;

    for line in code.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("impl ") && trimmed.contains(" for ") {
            let items = parse_rust_code(line);
            for item in items {
                if let ItemKind::ImplBlock {
                    trait_name: Some(trait_name),
                    ..
                } = item.kind
                {
                    current_trait = trait_name;
                    in_trait_impl = true;
                    depth = 0;
                }
            }
        }

        if in_trait_impl {
            depth += trimmed.matches('{').count();
            depth -= trimmed.matches('}').count();

            if trimmed.contains("fn ") && !current_trait.is_empty() {
                if let Some(body_start) = trimmed.find('{') {
                    let sig = trimmed[..body_start].trim().to_string() + ";";
                    if let Some(methods) = trait_methods.get_mut(&current_trait) {
                        methods.push(sig);
                    }
                }
            }

            if depth == 0 {
                in_trait_impl = false;
                current_trait.clear();
            }
        }
    }
}

pub fn inject_types_into_modules(code: &str, existing_types: &HashSet<String>) -> String {
    inject_types_recursive(code, existing_types)
}

fn inject_types_recursive(code: &str, existing_types: &HashSet<String>) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = code.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let items = parse_rust_code(line);

        let is_module = items
            .iter()
            .any(|item| matches!(item.kind, ItemKind::Module));

        if is_module {
            let (module_end, module_content, types_needed) =
                extract_module(i, &lines, existing_types);

            result.push_str(lines[i]);
            result.push('\n');

            if !types_needed.is_empty() {
                for type_name in &types_needed {
                    result.push_str(&format!("    use crate::{};\n", type_name));
                }
                result.push('\n');
            }

            result.push_str(&module_content);
            if !module_content.is_empty() && !module_content.ends_with('\n') {
                result.push('\n');
            }

            result.push_str(lines[module_end]);
            result.push('\n');

            i = module_end + 1;
        } else {
            result.push_str(line);
            result.push('\n');
            i += 1;
        }
    }

    result
}

fn extract_module(
    start: usize,
    lines: &[&str],
    existing_types: &HashSet<String>,
) -> (usize, String, Vec<String>) {
    let mut depth = 1;
    let mut j = start + 1;

    while j < lines.len() && depth > 0 {
        depth += lines[j].matches('{').count();
        depth -= lines[j].matches('}').count();
        j += 1;
    }

    let module_end = j - 1;
    let module_content_lines = &lines[(start + 1)..module_end];
    let module_content = module_content_lines.join("\n");

    let mut types_needed = Vec::new();
    for content_line in module_content_lines {
        let items = parse_rust_code(content_line);
        for item in items {
            if let ItemKind::ImplBlock {
                trait_name,
                type_name,
            } = item.kind
            {
                for name in [trait_name, Some(type_name)]
                    .iter()
                    .filter_map(|n| n.as_ref())
                {
                    if !existing_types.contains(name) && !types_needed.contains(name) {
                        types_needed.push(name.clone());
                    }
                }
            }
        }
    }

    let processed_content = inject_types_recursive(&module_content, existing_types);
    (module_end, processed_content, types_needed)
}

pub fn format_with_rustfmt(code: &str) -> Option<String> {
    use std::io::Write;

    let mut child = Command::new("rustfmt")
        .args(&["--edition", "2021"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .ok()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(code.as_bytes()).ok()?;
    }

    let output = child.wait_with_output().ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        eprintln!("rustfmt failed, using unformatted code");
        Some(code.to_string())
    }
}
