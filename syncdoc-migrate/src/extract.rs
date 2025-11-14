use proc_macro2::TokenStream;
use syncdoc_core::parse::Attribute;
use unsynn::*;

/// Extracts documentation content from a list of attributes
///
/// Returns the concatenated documentation strings if any doc attributes are found,
/// otherwise returns None.
pub fn extract_doc_content(attrs: &Option<Many<Attribute>>) -> Option<String> {
    let attrs = attrs.as_ref()?;

    let mut doc_strings = Vec::new();

    for attr_delimited in &attrs.0 {
        // Extract the actual Attribute from the Delimited wrapper
        if let Some(doc_content) = extract_from_single_attr(&attr_delimited.value) {
            doc_strings.push(doc_content);
        }
    }

    if doc_strings.is_empty() {
        None
    } else {
        Some(doc_strings.join("\n").trim().to_string())
    }
}

/// Helper to check if any doc attributes are present
pub fn has_doc_attrs(attrs: &Option<Many<Attribute>>) -> bool {
    extract_doc_content(attrs).is_some()
}

/// Checks if a BracketGroup contains a doc attribute
/// This properly parses the attribute content instead of string manipulation
pub fn is_doc_attribute_bracket(bracket: &BracketGroup) -> bool {
    // Extract the token stream from the bracket group
    let mut ts = TokenStream::new();
    unsynn::ToTokens::to_tokens(bracket, &mut ts);

    // Get the content inside the brackets
    let content = if let Some(proc_macro2::TokenTree::Group(g)) = ts.into_iter().next() {
        g.stream()
    } else {
        return false;
    };

    // Try to parse as tokens and check first ident
    let tokens: Vec<proc_macro2::TokenTree> = content.into_iter().collect();

    if let Some(proc_macro2::TokenTree::Ident(ident)) = tokens.first() {
        let ident_str = ident.to_string();
        // Only check the identifier itself
        ident_str == "doc" || ident_str == "cfg_attr"
    } else {
        false
    }
}

/// Checks if an outer attribute is a doc attribute
pub fn is_outer_doc_attr(attr: &syncdoc_core::parse::Attribute) -> bool {
    is_doc_attribute_bracket(&attr.content)
}

/// Extracts doc content from a single attribute
fn extract_from_single_attr(attr: &Attribute) -> Option<String> {
    let mut tokens = TokenStream::new();
    unsynn::ToTokens::to_tokens(attr, &mut tokens);

    let token_str = tokens.to_string();

    // Check if this is a doc attribute
    if !token_str.starts_with("# [") {
        return None;
    }

    // Look for doc = "..." pattern
    if let Some(doc_start) = token_str.find("doc") {
        let after_doc = &token_str[doc_start..];

        // Find the equals sign and opening quote
        if let Some(eq_pos) = after_doc.find('=') {
            let after_eq = &after_doc[eq_pos + 1..].trim_start();

            // Extract string content
            if let Some(content) = extract_string_literal(after_eq) {
                return Some(content);
            }
        }
    }

    None
}

/// Extracts a string literal from token text
fn extract_string_literal(s: &str) -> Option<String> {
    let s = s.trim();

    // Handle regular string "..."
    if s.starts_with('"') {
        if let Some(end_pos) = find_closing_quote(s, 1) {
            return Some(s[1..end_pos].to_string());
        }
    }

    // Handle raw string r#"..."#
    if s.starts_with("r#") {
        if let Some(start) = s.find('"') {
            if let Some(end) = s[start + 1..].find("\"#") {
                return Some(s[start + 1..start + 1 + end].to_string());
            }
        }
    }

    // Handle raw string r"..."
    if s.starts_with("r\"") {
        if let Some(end_pos) = find_closing_quote(s, 2) {
            return Some(s[2..end_pos].to_string());
        }
    }

    None
}

/// Finds the closing quote, accounting for escaped quotes
fn find_closing_quote(s: &str, start: usize) -> Option<usize> {
    let chars = s[start..].chars().enumerate();
    let mut escaped = false;

    for (i, ch) in chars {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => return Some(start + i),
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod extract_tests;
