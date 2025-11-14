// syncdoc-core/tests/helpers/parser.rs

use pest::Parser;
use pest_derive::Parser;
use std::collections::{HashMap, HashSet};

#[derive(Parser)]
#[grammar = "tests/helpers/rust.pest"]
pub struct RustParser;

#[derive(Debug, Clone)]
pub struct ParsedItem {
    pub kind: ItemKind,
    pub name: String,
    pub line: usize,
    pub has_generics: bool,
    pub module_path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    Function,
    Struct,
    Enum,
    Const,
    TypeAlias,
    Trait,
    Module,
    ImplBlock {
        trait_name: Option<String>,
        type_name: String,
    },
    Method {
        parent: String,
        trait_name: Option<String>,
    },
    Field {
        struct_name: String,
    },
}

pub fn parse_rust_code(code: &str) -> Vec<ParsedItem> {
    let mut items = Vec::new();

    for (line_no, line) in code.lines().enumerate() {
        if let Ok(pairs) = RustParser::parse(Rule::line, line) {
            for pair in pairs {
                extract_items(pair, line_no, &mut items, line);
            }
        }
    }

    items
}

fn extract_items(
    pair: pest::iterators::Pair<Rule>,
    line_no: usize,
    items: &mut Vec<ParsedItem>,
    line: &str,
) {
    match pair.as_rule() {
        Rule::function => {
            if let Some(name) = extract_name_from_inner(pair.clone()) {
                items.push(ParsedItem {
                    kind: ItemKind::Function,
                    name,
                    line: line_no,
                    has_generics: line.contains('<') && line.contains('>'),
                    module_path: Vec::new(),
                });
            }
        }
        Rule::struct_def => {
            if let Some(name) = extract_name_from_inner(pair.clone()) {
                items.push(ParsedItem {
                    kind: ItemKind::Struct,
                    name,
                    line: line_no,
                    has_generics: line.contains('<') && line.contains('>'),
                    module_path: Vec::new(),
                });
            }
        }
        Rule::enum_def => {
            if let Some(name) = extract_name_from_inner(pair.clone()) {
                items.push(ParsedItem {
                    kind: ItemKind::Enum,
                    name,
                    line: line_no,
                    has_generics: line.contains('<') && line.contains('>'),
                    module_path: Vec::new(),
                });
            }
        }
        Rule::const_def => {
            if let Some(name) = extract_name_from_inner(pair.clone()) {
                items.push(ParsedItem {
                    kind: ItemKind::Const,
                    name,
                    line: line_no,
                    has_generics: false,
                    module_path: Vec::new(),
                });
            }
        }
        Rule::type_def => {
            if let Some(name) = extract_name_from_inner(pair.clone()) {
                items.push(ParsedItem {
                    kind: ItemKind::TypeAlias,
                    name,
                    line: line_no,
                    has_generics: false,
                    module_path: Vec::new(),
                });
            }
        }
        Rule::trait_def => {
            if let Some(name) = extract_name_from_inner(pair.clone()) {
                items.push(ParsedItem {
                    kind: ItemKind::Trait,
                    name,
                    line: line_no,
                    has_generics: false,
                    module_path: Vec::new(),
                });
            }
        }
        Rule::module_def => {
            if let Some(name) = extract_name_from_inner(pair.clone()) {
                items.push(ParsedItem {
                    kind: ItemKind::Module,
                    name,
                    line: line_no,
                    has_generics: false,
                    module_path: Vec::new(),
                });
            }
        }
        Rule::impl_block => {
            extract_impl_block(pair.clone(), line_no, items, line);
        }
        Rule::field => {
            if let Some(name) = extract_name_from_inner(pair.clone()) {
                items.push(ParsedItem {
                    kind: ItemKind::Field {
                        struct_name: String::new(),
                    },
                    name,
                    line: line_no,
                    has_generics: false,
                    module_path: Vec::new(),
                });
            }
        }
        _ => {
            for inner in pair.into_inner() {
                extract_items(inner, line_no, items, line);
            }
        }
    }
}

fn extract_name_from_inner(pair: pest::iterators::Pair<Rule>) -> Option<String> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::ident {
            return Some(inner.as_str().to_string());
        }
    }
    None
}

fn extract_impl_block(
    pair: pest::iterators::Pair<Rule>,
    line_no: usize,
    items: &mut Vec<ParsedItem>,
    line: &str,
) {
    let mut trait_name: Option<String> = None;
    let mut type_name: Option<String> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::trait_impl => {
                let mut names = Vec::new();
                for part in inner.into_inner() {
                    if part.as_rule() == Rule::ident {
                        names.push(part.as_str().to_string());
                    }
                }
                if names.len() == 2 {
                    trait_name = Some(names[0].clone());
                    type_name = Some(names[1].clone());
                }
            }
            Rule::type_impl => {
                for part in inner.into_inner() {
                    if part.as_rule() == Rule::ident {
                        type_name = Some(part.as_str().to_string());
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(type_name) = &type_name {
        // Check if the type name is followed by generics in the line
        let has_type_generics = if let Some(pos) = line.find(type_name) {
            let after_type = &line[pos + type_name.len()..];
            after_type.trim_start().starts_with('<')
        } else {
            false
        };

        items.push(ParsedItem {
            kind: ItemKind::ImplBlock {
                trait_name: trait_name.clone(),
                type_name: type_name.clone(),
            },
            name: type_name.clone(),
            line: line_no,
            has_generics: has_type_generics,
            module_path: Vec::new(),
        });
    }
}

pub fn extract_existing_types(code: &str) -> HashSet<String> {
    let items = parse_rust_code(code);
    let mut types = HashSet::new();

    for item in items {
        match item.kind {
            ItemKind::Struct | ItemKind::Enum | ItemKind::Trait => {
                types.insert(item.name.clone());
                if item.has_generics {
                    types.insert(format!("{}<T>", item.name));
                }
            }
            _ => {}
        }
    }

    types
}

pub struct CodeStructure {
    pub items: Vec<ParsedItem>,
    pub modules: HashMap<String, Vec<ParsedItem>>,
    pub impl_blocks: HashMap<(Vec<String>, String), Vec<ParsedItem>>,
    pub struct_fields: HashMap<String, Vec<String>>,
    pub trait_methods: HashMap<String, Vec<String>>,
}

pub fn analyze_code_structure(code: &str) -> CodeStructure {
    let mut structure = CodeStructure {
        items: Vec::new(),
        modules: HashMap::new(),
        impl_blocks: HashMap::new(),
        struct_fields: HashMap::new(),
        trait_methods: HashMap::new(),
    };

    let mut current_context: Vec<String> = Vec::new();
    let mut brace_depth: i32 = 0;
    let mut in_module = false;
    let mut in_impl = false;
    let mut in_struct = false;
    let mut in_trait = false;
    let mut current_impl_name = String::new();
    let mut current_impl_trait: Option<String> = None;
    let mut current_struct_name = String::new();
    let mut current_trait_name = String::new();
    let mut module_depths: Vec<i32> = Vec::new(); // Track depth at which each module started

    for (line_no, line) in code.lines().enumerate() {
        let trimmed = line.trim();
        let parsed = parse_rust_code(line);

        // Track context
        for item in &parsed {
            match &item.kind {
                ItemKind::Module => {
                    current_context.push(item.name.clone());
                    module_depths.push(brace_depth);
                    in_module = true;
                    // Reset impl state when entering a new module
                    in_impl = false;
                    current_impl_name.clear();
                    current_impl_trait = None;
                }
                ItemKind::Trait => {
                    current_trait_name = item.name.clone();
                    in_trait = trimmed.contains('{');
                    structure.items.push(ParsedItem {
                        kind: item.kind.clone(),
                        name: item.name.clone(),
                        line: item.line,
                        has_generics: item.has_generics,
                        module_path: current_context.clone(),
                    });
                }
                ItemKind::Struct => {
                    if trimmed.contains('{') {
                        current_struct_name = item.name.clone();
                        in_struct = true;
                    }
                    if in_module && !current_context.is_empty() {
                        structure
                            .modules
                            .entry(current_context.join("::"))
                            .or_insert_with(Vec::new)
                            .push(ParsedItem {
                                kind: item.kind.clone(),
                                name: item.name.clone(),
                                line: item.line,
                                has_generics: item.has_generics,
                                module_path: current_context.clone(),
                            });
                    } else {
                        structure.items.push(ParsedItem {
                            kind: item.kind.clone(),
                            name: item.name.clone(),
                            line: item.line,
                            has_generics: item.has_generics,
                            module_path: current_context.clone(),
                        });
                    }
                }
                ItemKind::ImplBlock {
                    trait_name,
                    type_name,
                } => {
                    current_impl_name = type_name.clone();
                    current_impl_trait = trait_name.clone();
                    in_impl = true;
                }
                ItemKind::Function if in_impl && !current_impl_name.is_empty() => {
                    let method_item = ParsedItem {
                        kind: ItemKind::Method {
                            parent: current_impl_name.clone(),
                            trait_name: current_impl_trait.clone(),
                        },
                        name: item.name.clone(),
                        line: line_no,
                        has_generics: false,
                        module_path: current_context.clone(),
                    };
                    structure
                        .impl_blocks
                        .entry((current_context.clone(), current_impl_name.clone()))
                        .or_insert_with(Vec::new)
                        .push(method_item);
                }
                ItemKind::Function if in_trait && !current_trait_name.is_empty() => {
                    structure
                        .trait_methods
                        .entry(current_trait_name.clone())
                        .or_insert_with(Vec::new)
                        .push(item.name.clone());
                }
                ItemKind::Field { .. } if in_struct => {
                    structure
                        .struct_fields
                        .entry(current_struct_name.clone())
                        .or_insert_with(Vec::new)
                        .push(item.name.clone());
                }
                _ => {
                    if in_module && !current_context.is_empty() {
                        structure
                            .modules
                            .entry(current_context.join("::"))
                            .or_insert_with(Vec::new)
                            .push(ParsedItem {
                                kind: item.kind.clone(),
                                name: item.name.clone(),
                                line: item.line,
                                has_generics: item.has_generics,
                                module_path: current_context.clone(),
                            });
                    } else if !in_impl && !in_trait && !in_struct {
                        structure.items.push(ParsedItem {
                            kind: item.kind.clone(),
                            name: item.name.clone(),
                            line: item.line,
                            has_generics: item.has_generics,
                            module_path: current_context.clone(),
                        });
                    }
                }
            }
        }

        // Track braces carefully
        let opens = trimmed.matches('{').count() as i32;
        let closes = trimmed.matches('}').count() as i32;
        brace_depth += opens;

        // Process closes and check if we need to exit any contexts
        for _ in 0..closes {
            brace_depth -= 1;

            if brace_depth < 0 {
                brace_depth = 0;
            }

            // Check if we're exiting various contexts
            if in_impl && brace_depth <= 0 {
                in_impl = false;
                current_impl_name.clear();
                current_impl_trait = None;
            }
            if in_struct && brace_depth <= 0 {
                in_struct = false;
                current_struct_name.clear();
            }
            if in_trait && brace_depth <= 0 {
                in_trait = false;
                current_trait_name.clear();
            }

            // Check if we're exiting any modules
            while !module_depths.is_empty() && brace_depth <= *module_depths.last().unwrap() {
                module_depths.pop();
                current_context.pop();
            }

            in_module = !current_context.is_empty();
        }
    }

    structure
}
