use std::collections::HashSet;

use super::regex::{ENUM_RE, STRUCT_RE, TRAIT_RE};

pub fn extract_existing_types(code: &str) -> HashSet<String> {
    let mut types = HashSet::new();

    // Extract struct names
    for cap in STRUCT_RE.captures_iter(code) {
        if let Some(name) = cap.get(1) {
            let clean_name = name.as_str().to_string();
            types.insert(clean_name.clone());

            // Check if it's a generic definition (look ahead in the line)
            if let Some(line) = code[cap.get(0).unwrap().start()..].lines().next() {
                if line.contains('<') && line.contains('>') {
                    types.insert(format!("{}<T>", clean_name));
                }
            }
        }
    }

    // Extract trait names
    for cap in TRAIT_RE.captures_iter(code) {
        if let Some(name) = cap.get(1) {
            types.insert(name.as_str().to_string());
        }
    }

    // Extract enum names
    for cap in ENUM_RE.captures_iter(code) {
        if let Some(name) = cap.get(1) {
            types.insert(name.as_str().to_string());
        }
    }

    types
}
