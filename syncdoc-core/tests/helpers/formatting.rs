use itertools::Itertools;
use std::collections::{HashMap, HashSet};

use super::regex::{ENUM_RE, IMPL_RE, MOD_RE, STRUCT_RE, TRAIT_RE};

pub fn create_dummy_types_str(code: &str) -> (String, HashSet<String>) {
    let mut existing_types = HashSet::new();
    let mut types_to_define = Vec::new();
    let mut trait_methods: HashMap<String, Vec<String>> = HashMap::new();

    // Extract existing types
    for regex in [&*STRUCT_RE, &*ENUM_RE, &*TRAIT_RE] {
        for cap in regex.captures_iter(code) {
            if let Some(name) = cap.get(1).map(|m| m.as_str()) {
                existing_types.insert(name.to_string());
                if regex.as_str() == STRUCT_RE.as_str() {
                    if let Some(line) = code[cap.get(0).unwrap().start()..].lines().next() {
                        if line.contains('<') && line.contains('>') {
                            existing_types.insert(format!("{}<T>", name));
                        }
                    }
                }
            }
        }
    }

    // Process impl blocks for missing types
    for cap in IMPL_RE.captures_iter(code) {
        let trait_name = cap.get(1).map(|m| m.as_str());
        let type_name = cap.get(2).map(|m| m.as_str()).unwrap_or("");

        if let Some(trait_name) = trait_name {
            if !trait_name.is_empty()
                && trait_name.chars().next().unwrap().is_uppercase()
                && !existing_types.contains(trait_name)
            {
                trait_methods
                    .entry(trait_name.to_string())
                    .or_insert_with(Vec::new);
            }
        }

        if !type_name.is_empty()
            && type_name.chars().next().unwrap().is_uppercase()
            && !existing_types.contains(type_name)
        {
            let match_start = cap.get(0).unwrap().start();
            if let Some(line) = code[match_start..].lines().next() {
                let has_generics = line.contains('<') && line.contains('>');
                types_to_define.push(if has_generics {
                    format!(
                        "pub struct {}<T> {{ _inner: std::marker::PhantomData<T> }}",
                        type_name
                    )
                } else {
                    format!("pub struct {};", type_name)
                });
            }
        }
    }

    // Extract trait methods from impl blocks
    let mut in_trait_impl = false;
    let mut current_trait = String::new();
    let mut depth = 0;

    for line in code.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("impl ") && trimmed.contains(" for ") {
            if let Some(caps) = IMPL_RE.captures(trimmed) {
                if let Some(trait_name) = caps.get(1).map(|m| m.as_str()) {
                    // Only track if this trait needs to be defined
                    if !existing_types.contains(trait_name) {
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

            if trimmed.contains("fn ") && !current_trait.is_empty() {
                if let Some(body_start) = trimmed.find('{') {
                    let sig = trimmed[..body_start].trim().to_string() + ";";
                    trait_methods
                        .entry(current_trait.clone())
                        .or_insert_with(Vec::new)
                        .push(sig);
                }
            }

            if depth == 0 {
                in_trait_impl = false;
                current_trait.clear();
            }
        }
    }

    // Generate trait definitions (only for traits that aren't already defined)
    for (trait_name, methods) in trait_methods {
        types_to_define.push(if methods.is_empty() {
            format!("pub trait {} {{}}", trait_name)
        } else {
            format!(
                "pub trait {} {{\n    {}\n}}",
                trait_name,
                methods.join("\n    ")
            )
        });
    }

    (types_to_define.join("\n"), existing_types)
}

pub fn inject_types_into_modules(code: &str, existing_types: &HashSet<String>) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = code.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if MOD_RE.is_match(line.trim()) {
            let module_start = i;
            let mut depth = 1;
            let mut j = i + 1;

            while j < lines.len() && depth > 0 {
                depth += lines[j].matches('{').count();
                depth -= lines[j].matches('}').count();
                j += 1;
            }

            let module_end = j - 1;
            let module_content = lines[(module_start + 1)..module_end].join("\n");

            let types_needed: Vec<String> = lines[(module_start + 1)..module_end]
                .iter()
                .filter_map(|content_line| {
                    IMPL_RE.captures(content_line.trim()).and_then(|caps| {
                        caps.get(1)
                            .or(caps.get(2))
                            .map(|m| m.as_str())
                            .filter(|&type_name| {
                                !type_name.is_empty() && !existing_types.contains(type_name)
                            })
                            .map(|s| s.to_string())
                    })
                })
                .unique()
                .collect();

            let processed_content = inject_types_into_modules(&module_content, existing_types);

            result.push_str(lines[module_start]);
            result.push('\n');

            if !types_needed.is_empty() {
                for type_name in &types_needed {
                    result.push_str(&format!("    use crate::{};\n", type_name));
                }
                result.push('\n');
            }

            result.push_str(&processed_content);
            if !processed_content.is_empty() && !processed_content.ends_with('\n') {
                result.push('\n');
            }

            result.push_str(lines[module_end]);
            result.push('\n');

            i = j;
        } else {
            result.push_str(line);
            result.push('\n');
            i += 1;
        }
    }

    result
}

pub fn format_with_rustfmt(code: &str) -> Option<String> {
    use duct::cmd;
    cmd!("rustfmt", "--edition", "2021")
        .stdin_bytes(code)
        .read()
        .ok()
        .or_else(|| Some(code.to_string()))
}
