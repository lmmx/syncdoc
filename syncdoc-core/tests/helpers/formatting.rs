use std::collections::{HashMap, HashSet};
use std::process::Command;

pub fn create_dummy_types_str(code: &str, existing_types: &HashSet<String>) -> String {
    let mut types_to_define = Vec::new();
    let mut trait_methods: HashMap<String, Vec<String>> = HashMap::new();

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

    // Scan for methods inside trait impls to add to trait definitions
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

pub fn inject_types_into_modules(code: &str, existing_types: &HashSet<String>) -> String {
    inject_types_recursive(code, existing_types)
}

fn inject_types_recursive(code: &str, existing_types: &HashSet<String>) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = code.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Check if this line starts a module
        if (trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ")) && trimmed.contains('{')
        {
            let module_start = i;
            let mut depth = 1;
            let mut j = i + 1;

            // Find the end of this module
            while j < lines.len() && depth > 0 {
                let module_line = lines[j];
                depth += module_line.matches('{').count();
                depth -= module_line.matches('}').count();
                j += 1;
            }

            let module_end = j - 1;

            // Extract module content
            let module_content_lines = &lines[(module_start + 1)..module_end];
            let module_content = module_content_lines.join("\n");

            // Find types needed in this module
            let mut types_needed = Vec::new();
            for content_line in module_content_lines {
                let trimmed_content = content_line.trim();
                if trimmed_content.starts_with("impl ") || trimmed_content.starts_with("impl<") {
                    let impl_part = trimmed_content.split('{').next().unwrap_or("");

                    let type_name = if let Some(for_pos) = impl_part.find(" for ") {
                        let type_part = impl_part[for_pos + 5..].trim();
                        type_part
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .split('<')
                            .next()
                            .unwrap_or("")
                            .trim()
                    } else {
                        let parts: Vec<&str> = impl_part.split_whitespace().collect();
                        parts
                            .last()
                            .map(|s| s.split('<').next().unwrap_or(s).trim())
                            .unwrap_or("")
                    };

                    if !type_name.is_empty()
                        && !existing_types.contains(type_name)
                        && !types_needed.contains(&type_name.to_string())
                    {
                        types_needed.push(type_name.to_string());
                    }
                }
            }

            // Recursively process module content
            let processed_content = inject_types_recursive(&module_content, existing_types);

            // Write module with injected imports
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
