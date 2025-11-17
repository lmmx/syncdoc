use super::*;
use braces::{brace_paths, BraceConfig};
use insta::assert_snapshot;
use quote::quote;

fn to_braces(paths: &[&str]) -> String {
    let braces_config = BraceConfig::default();
    brace_paths(paths, &braces_config).expect("Brace error")
}

// Mock version of TokenProcessor that collects paths instead of calling omnidoc_impl
struct PathCollector {
    base_path: String,
    context: Vec<String>,
    collected_paths: Vec<String>,
}

impl PathCollector {
    fn new(base_path: String) -> Self {
        Self {
            base_path,
            context: Vec::new(),
            collected_paths: Vec::new(),
        }
    }

    fn collect_paths(mut self, input: TokenStream) -> Vec<String> {
        match input.into_token_iter().parse::<ModuleContent>() {
            Ok(content) => {
                for item in content.items.0 {
                    self.collect_from_item(item.value);
                }
            }
            Err(_) => {}
        }
        self.collected_paths.sort();
        self.collected_paths
    }

    fn collect_from_item(&mut self, item: ModuleItem) {
        match item {
            ModuleItem::TraitMethod(method_sig) => {
                self.add_path(&method_sig.name.to_string());
            }
            ModuleItem::Function(func_sig) => {
                self.add_path(&func_sig.name.to_string());
            }
            ModuleItem::ImplBlock(impl_block) => {
                self.collect_from_impl_block(impl_block);
            }
            ModuleItem::Module(module) => {
                self.collect_from_module(module);
            }
            ModuleItem::Trait(trait_def) => {
                self.collect_from_trait(trait_def);
            }
            ModuleItem::Enum(enum_sig) => {
                self.collect_from_enum(enum_sig);
            }
            ModuleItem::Struct(struct_sig) => {
                self.collect_from_struct(struct_sig);
            }
            ModuleItem::TypeAlias(type_alias) => {
                self.add_path(&type_alias.name.to_string());
            }
            ModuleItem::Const(const_sig) => {
                self.add_path(&const_sig.name.to_string());
            }
            ModuleItem::Static(static_sig) => {
                self.add_path(&static_sig.name.to_string());
            }
            ModuleItem::Other(_) => {}
        }
    }

    fn collect_from_impl_block(&mut self, impl_block: ImplBlockSig) {
        let context_path = if let Some(for_trait) = &impl_block.for_trait {
            let trait_name = extract_type_name(&impl_block.target_type);
            let type_name = extract_first_ident_from_tokens(&for_trait.second);
            vec![type_name, trait_name]
        } else {
            let type_name = extract_type_name(&impl_block.target_type);
            vec![type_name]
        };

        self.context.extend(context_path);

        let module_content = &impl_block.items.content;
        for item_delimited in &module_content.items.0 {
            self.collect_from_item(item_delimited.value.clone());
        }

        // Pop context
        let context_len = if impl_block.for_trait.is_some() { 2 } else { 1 };
        for _ in 0..context_len {
            self.context.pop();
        }
    }

    fn collect_from_module(&mut self, module: ModuleSig) {
        self.context.push(module.name.to_string());

        let module_content = &module.items.content;
        for item_delimited in &module_content.items.0 {
            self.collect_from_item(item_delimited.value.clone());
        }

        self.context.pop();
    }

    fn collect_from_trait(&mut self, trait_def: TraitSig) {
        let trait_name = trait_def.name.to_string();
        self.add_path(&trait_name);

        self.context.push(trait_name);

        let trait_content = &trait_def.items.content;
        for item_delimited in &trait_content.items.0 {
            self.collect_from_item(item_delimited.value.clone());
        }

        self.context.pop();
    }

    fn collect_from_enum(&mut self, enum_sig: crate::parse::EnumSig) {
        let enum_name = enum_sig.name.to_string();
        self.add_path(&enum_name);

        if let Some(variants_cdv) = enum_sig.variants.content.as_ref() {
            for variant_delimited in &variants_cdv.0 {
                let variant = &variant_delimited.value;
                let variant_name = variant.name.to_string();

                // Add path for variant itself
                self.add_path_with_context(&enum_name, &variant_name);

                // If struct-valued, add paths for fields
                if let Some(crate::parse::EnumVariantData::Struct(fields_containing)) =
                    &variant.data
                {
                    if let Some(fields_cdv) = fields_containing.content.as_ref() {
                        for field_delimited in &fields_cdv.0 {
                            let field = &field_delimited.value;
                            let field_name = field.name.to_string();
                            self.add_path_with_nested_context(
                                &enum_name,
                                &variant_name,
                                &field_name,
                            );
                        }
                    }
                }
            }
        }
    }

    fn collect_from_struct(&mut self, struct_sig: crate::parse::StructSig) {
        let struct_name = struct_sig.name.to_string();
        self.add_path(&struct_name);

        if let crate::parse::StructBody::Named(fields_containing) = &struct_sig.body {
            if let Some(fields_cdv) = fields_containing.content.as_ref() {
                for field_delimited in &fields_cdv.0 {
                    let field = &field_delimited.value;
                    let field_name = field.name.to_string();
                    self.add_path_with_context(&struct_name, &field_name);
                }
            }
        }
    }

    fn add_path(&mut self, name: &str) {
        let mut path_parts = vec![self.base_path.clone()];
        path_parts.extend(self.context.iter().cloned());
        path_parts.push(format!("{}.md", name));
        self.collected_paths.push(path_parts.join("/"));
    }

    fn add_path_with_context(&mut self, parent: &str, name: &str) {
        let mut path_parts = vec![self.base_path.clone()];
        path_parts.extend(self.context.iter().cloned());
        path_parts.push(format!("{}/{}.md", parent, name));
        self.collected_paths.push(path_parts.join("/"));
    }

    fn add_path_with_nested_context(&mut self, grandparent: &str, parent: &str, name: &str) {
        let mut path_parts = vec![self.base_path.clone()];
        path_parts.extend(self.context.iter().cloned());
        path_parts.push(format!("{}/{}/{}.md", grandparent, parent, name));
        self.collected_paths.push(path_parts.join("/"));
    }
}

#[test]
fn test_struct_valued_enum_variant_doc_paths() {
    let input = quote! {
        pub enum NodeType {
            Directory {
                name: String,
                path: String,
            },
            File {
                name: String,
                path: String,
            },
            Section(Section),
        }
    };

    let collector = PathCollector::new("docs".to_string());
    let paths = collector.collect_paths(input);
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();

    assert_snapshot!(to_braces(&path_refs), @"docs/NodeType/{Directory/{name,path,},File/{name,path,},Section,}.md");
}

#[test]
fn test_tuple_enum_variant_doc_paths() {
    let input = quote! {
        pub enum Result<T, E> {
            Ok(T),
            Err(E),
        }
    };

    let collector = PathCollector::new("docs".to_string());
    let paths = collector.collect_paths(input);
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();

    assert_snapshot!(to_braces(&path_refs), @"docs/Result/{Err,Ok,}.md");
}

#[test]
fn test_unit_enum_variant_doc_paths() {
    let input = quote! {
        pub enum ChunkType {
            Added,
            Deleted,
            Modified,
            Unchanged,
        }
    };

    let collector = PathCollector::new("docs".to_string());
    let paths = collector.collect_paths(input);
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();

    assert_snapshot!(to_braces(&path_refs), @"docs/ChunkType/{Added,Deleted,Modified,Unchanged,}.md");
}

#[test]
fn test_mixed_enum_variants_doc_paths() {
    let input = quote! {
        pub enum Message {
            Quit,
            Move { x: i32, y: i32 },
            Write(String),
            ChangeColor(i32, i32, i32),
        }
    };

    let collector = PathCollector::new("docs".to_string());
    let paths = collector.collect_paths(input);
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();

    assert_snapshot!(to_braces(&path_refs), @"docs/Message/{ChangeColor,Move/{x,y,},Quit,Write,}.md");
}

#[test]
fn test_trait_method_doc_paths() {
    let input = quote! {
        pub trait Display {
            fn fmt(&self, f: &mut Formatter) -> Result;
            fn fmt_debug(&self) -> String;
        }
    };

    let collector = PathCollector::new("docs".to_string());
    let paths = collector.collect_paths(input);
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();

    assert_snapshot!(to_braces(&path_refs), @"docs/Display/{fmt,fmt_debug,}.md");
}

#[test]
fn test_nested_context_doc_paths() {
    let input = quote! {
        pub mod outer {
            pub struct Container {
                field1: String,
            }
        }
    };

    let collector = PathCollector::new("docs".to_string());
    let paths = collector.collect_paths(input);
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();

    assert_snapshot!(to_braces(&path_refs), @"docs/outer/Container/{field1,}.md");
}

#[test]
fn test_struct_fields_doc_paths() {
    let input = quote! {
        pub struct Section {
            pub title: String,
            pub level: usize,
            pub line_start: i64,
        }
    };

    let collector = PathCollector::new("docs".to_string());
    let paths = collector.collect_paths(input);
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();

    assert_snapshot!(to_braces(&path_refs), @"docs/Section/{level,line_start,title,}.md");
}

#[test]
fn test_impl_block_methods_doc_paths() {
    let input = quote! {
        impl TreeNode {
            pub fn directory(name: String, path: String) -> Self {}
            pub fn file(name: String, path: String) -> Self {}
            pub fn section(section: Section) -> Self {}
        }
    };

    let collector = PathCollector::new("docs".to_string());
    let paths = collector.collect_paths(input);
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();

    assert_snapshot!(to_braces(&path_refs), @"docs/TreeNode/{directory.md,file.md,section.md}");
}

#[test]
fn test_trait_impl_methods_doc_paths() {
    let input = quote! {
        impl Display for TreeNode {
            fn fmt(&self, f: &mut Formatter) -> Result {}
        }
    };

    let collector = PathCollector::new("docs".to_string());
    let paths = collector.collect_paths(input);
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();

    assert_snapshot!(to_braces(&path_refs), @"docs/TreeNode/Display/fmt.md");
}
