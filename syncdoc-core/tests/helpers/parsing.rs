use std::collections::HashSet;

pub fn extract_existing_types(code: &str) -> HashSet<String> {
    let mut types = HashSet::new();

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
