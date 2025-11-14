// syncdoc-core/tests/helpers/docs.rs

use crate::TestCrate;
use super::parser::{ItemKind, analyze_code_structure};
use std::fs;

pub fn auto_create_docs(test_crate: &TestCrate, code: &str) {
    test_crate.write_doc("lib.md", "Test library");

    let structure = analyze_code_structure(code);

    // Create docs for top-level items
    for item in &structure.items {
        create_doc_for_item(test_crate, item);
    }

    // Create docs for module items
    for (_module_path, items) in &structure.modules {
        for item in items {
            create_doc_for_item(test_crate, item);
        }
    }

    // Create docs for impl block methods
    for ((module_path, _type_name), methods) in &structure.impl_blocks {
        for method in methods {
            if let ItemKind::Method { parent, trait_name } = &method.kind {
                let mut path_parts = vec!["lib".to_string()];
                path_parts.extend(module_path.clone());
                path_parts.push(parent.clone());

                if let Some(trait_name) = trait_name {
                    path_parts.push(trait_name.clone());
                }

                path_parts.push(format!("{}.md", method.name));
                let path = path_parts.join("/");

                let dir_path = test_crate.root().join("docs").join(path.trim_end_matches(".md"));
                fs::create_dir_all(dir_path.parent().unwrap()).ok();

                eprintln!("Creating doc for method: {}::{}", parent, method.name);
                test_crate.write_doc(&path, "Documentation for method");
            }
        }
    }

    // Create docs for trait methods
    for (trait_name, methods) in &structure.trait_methods {
        for method_name in methods {
            let path = format!("lib/{}/{}.md", trait_name, method_name);
            let dir_path = test_crate.root().join("docs").join(format!("lib/{}", trait_name));
            fs::create_dir_all(&dir_path).ok();

            eprintln!("Creating doc for trait method: {}::{}", trait_name, method_name);
            test_crate.write_doc(&path, &format!("Documentation for {}::{}", trait_name, method_name));
        }
    }

    // Handle struct fields
    for (struct_name, fields) in &structure.struct_fields {
        for field_name in fields {
            let path = format!("lib/{}/{}.md", struct_name, field_name);
            let dir_path = test_crate.root().join("docs").join(format!("lib/{}", struct_name));
            fs::create_dir_all(&dir_path).ok();

            eprintln!("Creating doc for field: {}::{}", struct_name, field_name);
            test_crate.write_doc(&path, "Documentation for field");
        }
    }
}

fn create_doc_for_item(test_crate: &TestCrate, item: &super::parser::ParsedItem) {
    let mut path_parts = vec!["lib"];

    // Build path from module_path
    for module_name in &item.module_path {
        path_parts.push(module_name.as_str());
    }

    match &item.kind {
        ItemKind::Function => {
            eprintln!("Creating doc for function: {}", item.name);
            path_parts.push(&item.name);
            test_crate.write_doc(
                &format!("{}.md", path_parts.join("/")),
                &format!("Documentation for {}", item.name),
            );
        }
        ItemKind::Struct => {
            eprintln!("Creating doc for struct: {}", item.name);
            path_parts.push(&item.name);
            test_crate.write_doc(
                &format!("{}.md", path_parts.join("/")),
                &format!("Documentation for {}", item.name),
            );
        }
        ItemKind::Enum => {
            eprintln!("Creating doc for enum: {}", item.name);
            path_parts.push(&item.name);
            test_crate.write_doc(
                &format!("{}.md", path_parts.join("/")),
                &format!("Documentation for {}", item.name),
            );
        }
        ItemKind::Const => {
            eprintln!("Creating doc for const: {}", item.name);
            path_parts.push(&item.name);
            test_crate.write_doc(
                &format!("{}.md", path_parts.join("/")),
                &format!("Documentation for {}", item.name),
            );
        }
        ItemKind::TypeAlias => {
            eprintln!("Creating doc for type alias: {}", item.name);
            path_parts.push(&item.name);
            test_crate.write_doc(
                &format!("{}.md", path_parts.join("/")),
                &format!("Documentation for {}", item.name),
            );
        }
        ItemKind::Trait => {
            eprintln!("Creating doc for trait: {}", item.name);
            let dir_path = test_crate
                .root()
                .join("docs")
                .join(path_parts.join("/"))
                .join(&item.name);
            fs::create_dir_all(&dir_path).ok();
            path_parts.push(&item.name);
            test_crate.write_doc(
                &format!("{}.md", path_parts.join("/")),
                &format!("Documentation for {}", item.name),
            );
        }
        ItemKind::Module => {
            eprintln!("Creating doc for module: {}", item.name);
            path_parts.push(&item.name);
            let dir_path = test_crate.root().join("docs").join(path_parts.join("/"));
            fs::create_dir_all(&dir_path).ok();
            test_crate.write_doc(
                &format!("{}.md", path_parts.join("/")),
                "Documentation for module",
            );
        }
        _ => {}
    }
}
