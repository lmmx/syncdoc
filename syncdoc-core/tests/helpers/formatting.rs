use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::LazyLock;

static IMPL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"impl(?:<[^>]+>)?\s+(?:(\w+)\s+for\s+)?(\w+)").unwrap());

pub fn create_dummy_types_str(code: &str, existing_types: &HashSet<String>) -> String {
    let mut types_to_define = Vec::new();
    let mut trait_methods: HashMap<String, Vec<String>> = HashMap::new();

    for cap in IMPL_RE.captures_iter(code) {
        let trait_name = cap.get(1).map(|m| m.as_str());
        let type_name = cap.get(2).map(|m| m.as_str()).unwrap_or("");

        // Handle trait name
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

        // Handle type name
        if !type_name.is_empty()
            && type_name.chars().next().unwrap().is_uppercase()
            && !existing_types.contains(type_name)
        {
            // Check if generic (look at the original match context)
            let match_start = cap.get(0).unwrap().start();
            if let Some(line) = code[match_start..].lines().next() {
                let has_generics = line.contains('<') && line.contains('>');
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

    // Scan for methods inside trait impls
    let lines: Vec<&str> = code.lines().collect();
    let mut in_trait_impl = false;
    let mut current_trait = String::new();
    let mut depth = 0;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.starts_with("impl ") && trimmed.contains(" for ") {
            if let Some(caps) = IMPL_RE.captures(trimmed) {
                if let Some(trait_name) = caps.get(1) {
                    current_trait = trait_name.as_str().to_string();
                    in_trait_impl = true;
                    depth = 0;
                }
            }
        }

        if in_trait_impl {
            depth += trimmed.matches('{').count();
            depth -= trimmed.matches('}').count();

            // Extract method signatures
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

    // Generate trait definitions
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

pub fn inject_types_into_modules(code: &str, existing_types: &HashSet<String>) -> String {
    inject_types_recursive(code, existing_types)
}

fn inject_types_recursive(code: &str, existing_types: &HashSet<String>) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = code.lines().collect();
    let mut i = 0;

    static MOD_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?:pub\s+)?mod\s+\w+\s*\{").unwrap());

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Check if this line starts a module
        if MOD_RE.is_match(trimmed) {
            let module_start = i;
            let mut depth = 1;
            let mut j = i + 1;

            while j < lines.len() && depth > 0 {
                let module_line = lines[j];
                depth += module_line.matches('{').count();
                depth -= module_line.matches('}').count();
                j += 1;
            }

            let module_end = j - 1;
            let module_content_lines = &lines[(module_start + 1)..module_end];
            let module_content = module_content_lines.join("\n");

            // Find types needed
            let mut types_needed = Vec::new();
            for content_line in module_content_lines {
                if let Some(caps) = IMPL_RE.captures(content_line.trim()) {
                    let type_name = caps
                        .get(1)
                        .or(caps.get(2))
                        .map(|m| m.as_str())
                        .unwrap_or("");

                    if !type_name.is_empty()
                        && !existing_types.contains(type_name)
                        && !types_needed.contains(&type_name.to_string())
                    {
                        types_needed.push(type_name.to_string());
                    }
                }
            }

            let processed_content = inject_types_recursive(&module_content, existing_types);

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
