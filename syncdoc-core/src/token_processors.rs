use crate::{inject_doc_attr, syncdoc_impl};
use proc_macro2::TokenStream;
use unsynn::*;

use crate::parse::{ImplBlockSig, ModuleContent, ModuleItem, ModuleSig, TraitSig};

pub struct TokenProcessor {
    input: TokenStream,
    base_path: String,
    cfg_attr: Option<String>,
    context: Vec<String>,
}

impl TokenProcessor {
    pub fn new(input: TokenStream, base_path: String, cfg_attr: Option<String>) -> Self {
        Self {
            input,
            base_path,
            cfg_attr,
            context: Vec::new(),
        }
    }

    pub fn process(self) -> TokenStream {
        match self
            .input
            .clone()
            .into_token_iter()
            .parse::<ModuleContent>()
        {
            Ok(_parsed) => self.process_module_content(),
            Err(_) => {
                // Fallback: if declarative parsing fails, use original input
                self.input
            }
        }
    }

    fn process_module_content(&self) -> TokenStream {
        let mut output = TokenStream::new();

        let content = match self
            .input
            .clone()
            .into_token_iter()
            .parse::<ModuleContent>()
        {
            Ok(c) => c,
            Err(_) => return self.input.clone(),
        };

        for item in content.items.0 {
            let processed_item = self.process_module_item(item.value);
            output.extend(processed_item);
        }

        output
    }

    fn process_module_item(&self, item: ModuleItem) -> TokenStream {
        match item {
            ModuleItem::TraitMethod(method_sig) => {
                let mut method_tokens = TokenStream::new();
                quote::ToTokens::to_tokens(&method_sig, &mut method_tokens);
                self.inject_doc_into_simple_item(method_tokens, &method_sig.name.to_string())
            }
            ModuleItem::Function(func_sig) => {
                let mut func_tokens = TokenStream::new();
                quote::ToTokens::to_tokens(&func_sig, &mut func_tokens);
                self.inject_doc_into_item(func_tokens, &func_sig.name.to_string())
            }
            ModuleItem::ImplBlock(impl_block) => self.process_impl_block(impl_block),
            ModuleItem::Module(module) => self.process_module_block(module),
            ModuleItem::Trait(trait_def) => self.process_trait_block(trait_def),
            ModuleItem::Enum(enum_sig) => self.process_enum(enum_sig),
            ModuleItem::Struct(struct_sig) => self.process_struct(struct_sig),
            ModuleItem::TypeAlias(type_alias) => {
                let mut alias_tokens = TokenStream::new();
                quote::ToTokens::to_tokens(&type_alias, &mut alias_tokens);
                self.inject_doc_into_simple_item(alias_tokens, &type_alias.name.to_string())
            }
            ModuleItem::Const(const_sig) => {
                let mut const_tokens = TokenStream::new();
                quote::ToTokens::to_tokens(&const_sig, &mut const_tokens);
                self.inject_doc_into_simple_item(const_tokens, &const_sig.name.to_string())
            }
            ModuleItem::Static(static_sig) => {
                let mut static_tokens = TokenStream::new();
                quote::ToTokens::to_tokens(&static_sig, &mut static_tokens);
                self.inject_doc_into_simple_item(static_tokens, &static_sig.name.to_string())
            }
            ModuleItem::Other(token) => {
                let mut tokens = TokenStream::new();
                token.to_tokens(&mut tokens);
                tokens
            }
        }
    }

    fn process_impl_block(&self, impl_block: ImplBlockSig) -> TokenStream {
        // Check if this is a trait impl (has "for" clause)
        let context_path = if let Some(for_trait) = &impl_block.for_trait {
            // This is "impl Trait for Type"
            // target_type contains the TRAIT name (before "for")
            let trait_name = extract_type_name(&impl_block.target_type);
            // for_trait contains "for Type" - extract Type
            let type_name = extract_first_ident_from_tokens(&for_trait.second);
            // Context should be: Type/Trait
            vec![type_name, trait_name]
        } else {
            // This is "impl Type"
            // target_type is the type being implemented
            let type_name = extract_type_name(&impl_block.target_type);
            vec![type_name]
        };

        // Create new processor with updated context
        let mut new_context = self.context.clone();
        new_context.extend(context_path);

        // Access parsed items directly
        let module_content = &impl_block.items.content;

        let new_processor = TokenProcessor {
            input: TokenStream::new(),
            base_path: self.base_path.clone(),
            cfg_attr: self.cfg_attr.clone(),
            context: new_context,
        };

        let mut processed_content = TokenStream::new();
        for item_delimited in &module_content.items.0 {
            processed_content
                .extend(new_processor.process_module_item(item_delimited.value.clone()));
        }

        // Reconstruct impl block
        let mut output = TokenStream::new();
        if let Some(attrs) = impl_block.attributes {
            for attr in attrs.0 {
                attr.to_tokens(&mut output);
            }
        }
        impl_block._impl.to_tokens(&mut output);
        if let Some(generics) = impl_block.generics {
            generics.to_tokens(&mut output);
        }
        for item in impl_block.target_type.0 {
            item.value.second.to_tokens(&mut output);
        }
        if let Some(for_part) = impl_block.for_trait {
            for_part.to_tokens(&mut output);
        }
        if let Some(where_clause) = impl_block.where_clause {
            where_clause.to_tokens(&mut output);
        }

        // Wrap processed content in braces
        let group = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, processed_content);
        output.extend(std::iter::once(proc_macro2::TokenTree::Group(group)));

        output
    }

    fn process_module_block(&self, module: ModuleSig) -> TokenStream {
        let mut new_context = self.context.clone();
        new_context.push(module.name.to_string());

        // Access parsed items directly
        let module_content = &module.items.content;

        let new_processor = TokenProcessor {
            input: TokenStream::new(),
            base_path: self.base_path.clone(),
            cfg_attr: self.cfg_attr.clone(),
            context: new_context,
        };

        let mut processed_content = TokenStream::new();
        for item_delimited in &module_content.items.0 {
            processed_content
                .extend(new_processor.process_module_item(item_delimited.value.clone()));
        }

        // Reconstruct module
        let mut output = TokenStream::new();
        if let Some(attrs) = module.attributes {
            for attr in attrs.0 {
                attr.to_tokens(&mut output);
            }
        }
        if let Some(vis) = module.visibility {
            vis.to_tokens(&mut output);
        }
        module._mod.to_tokens(&mut output);
        module.name.to_tokens(&mut output);

        let group = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, processed_content);
        output.extend(std::iter::once(proc_macro2::TokenTree::Group(group)));

        output
    }

    fn process_trait_block(&self, trait_def: TraitSig) -> TokenStream {
        let mut new_context = self.context.clone();
        new_context.push(trait_def.name.to_string());

        // Access parsed items directly
        let trait_content = &trait_def.items.content;

        let new_processor = TokenProcessor {
            input: TokenStream::new(),
            base_path: self.base_path.clone(),
            cfg_attr: self.cfg_attr.clone(),
            context: new_context,
        };

        let mut processed_content = TokenStream::new();
        for item_delimited in &trait_content.items.0 {
            processed_content
                .extend(new_processor.process_module_item(item_delimited.value.clone()));
        }

        // Inject doc for trait itself
        let mut output = TokenStream::new();
        if let Some(attrs) = trait_def.attributes {
            for attr in attrs.0 {
                attr.to_tokens(&mut output);
            }
        }
        if let Some(vis) = trait_def.visibility {
            vis.to_tokens(&mut output);
        }
        if let Some(unsafe_kw) = trait_def.unsafe_kw {
            unsafe_kw.to_tokens(&mut output);
        }
        trait_def._trait.to_tokens(&mut output);
        trait_def.name.to_tokens(&mut output);
        if let Some(generics) = trait_def.generics {
            generics.to_tokens(&mut output);
        }
        if let Some(bounds) = trait_def.bounds {
            bounds.to_tokens(&mut output);
        }
        if let Some(where_clause) = trait_def.where_clause {
            where_clause.to_tokens(&mut output);
        }

        let trait_name = trait_def.name.to_string();
        let trait_with_doc = self.inject_doc_into_simple_item(output, &trait_name);

        // Combine with processed body
        let mut final_output = trait_with_doc;
        let group = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, processed_content);
        final_output.extend(std::iter::once(proc_macro2::TokenTree::Group(group)));

        final_output
    }

    fn process_struct(&self, struct_sig: crate::parse::StructSig) -> TokenStream {
        let struct_name = struct_sig.name.to_string();

        // Process struct body for named fields
        let processed_body = match &struct_sig.body {
            crate::parse::StructBody::Named(fields_containing) => {
                if let Some(fields_cdv) = fields_containing.content.as_ref() {
                    let processed_fields = self.process_struct_fields(fields_cdv, &struct_name);
                    let group =
                        proc_macro2::Group::new(proc_macro2::Delimiter::Brace, processed_fields);
                    let mut ts = TokenStream::new();
                    ts.extend(std::iter::once(proc_macro2::TokenTree::Group(group)));
                    ts
                } else {
                    let mut ts = TokenStream::new();
                    unsynn::ToTokens::to_tokens(fields_containing, &mut ts);
                    ts
                }
            }
            other => {
                let mut ts = TokenStream::new();
                quote::ToTokens::to_tokens(other, &mut ts);
                ts
            }
        };

        // Reconstruct struct
        let mut output = TokenStream::new();
        if let Some(attrs) = struct_sig.attributes {
            for attr in attrs.0 {
                attr.to_tokens(&mut output);
            }
        }
        if let Some(vis) = struct_sig.visibility {
            vis.to_tokens(&mut output);
        }
        struct_sig._struct.to_tokens(&mut output);
        struct_sig.name.to_tokens(&mut output);
        if let Some(generics) = struct_sig.generics {
            generics.to_tokens(&mut output);
        }
        if let Some(where_clause) = struct_sig.where_clause {
            where_clause.to_tokens(&mut output);
        }

        let struct_with_doc = self.inject_doc_into_simple_item(output, &struct_name);
        let mut final_output = struct_with_doc;
        final_output.extend(processed_body);

        final_output
    }

    fn process_struct_fields(
        &self,
        fields_cdv: &CommaDelimitedVec<crate::parse::StructField>,
        struct_name: &str,
    ) -> TokenStream {
        let mut output = TokenStream::new();

        for (idx, field_delimited) in fields_cdv.0.iter().enumerate() {
            let field = &field_delimited.value;
            let field_name = field.name.to_string();

            let mut field_tokens = TokenStream::new();
            quote::ToTokens::to_tokens(field, &mut field_tokens);

            let documented =
                self.inject_doc_for_struct_field(field_tokens, struct_name, &field_name);
            output.extend(documented);

            if idx < fields_cdv.0.len() - 1 {
                output.extend(quote::quote! { , });
            }
        }

        output
    }

    fn inject_doc_for_struct_field(
        &self,
        field_tokens: TokenStream,
        struct_name: &str,
        field_name: &str,
    ) -> TokenStream {
        let mut path_parts = vec![self.base_path.clone()];
        path_parts.extend(self.context.iter().cloned());
        path_parts.push(format!("{}/{}.md", struct_name, field_name));

        let full_path = path_parts.join("/");
        inject_doc_attr(full_path, self.cfg_attr.clone(), field_tokens)
    }

    fn process_enum(&self, enum_sig: crate::parse::EnumSig) -> TokenStream {
        let enum_name = enum_sig.name.to_string();

        // Process enum variants
        let processed_variants = if let Some(variants_cdv) = enum_sig.variants.content.as_ref() {
            self.process_enum_variants(variants_cdv, &enum_name)
        } else {
            TokenStream::new()
        };

        // Reconstruct enum
        let mut output = TokenStream::new();
        if let Some(attrs) = enum_sig.attributes {
            for attr in attrs.0 {
                attr.to_tokens(&mut output);
            }
        }
        if let Some(vis) = enum_sig.visibility {
            vis.to_tokens(&mut output);
        }
        enum_sig._enum.to_tokens(&mut output);
        enum_sig.name.to_tokens(&mut output);
        if let Some(generics) = enum_sig.generics {
            generics.to_tokens(&mut output);
        }
        if let Some(where_clause) = enum_sig.where_clause {
            where_clause.to_tokens(&mut output);
        }

        let enum_with_doc = self.inject_doc_into_simple_item(output, &enum_name);
        let mut final_output = enum_with_doc;
        let group = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, processed_variants);
        final_output.extend(std::iter::once(proc_macro2::TokenTree::Group(group)));

        final_output
    }

    fn process_enum_variants(
        &self,
        variants_cdv: &CommaDelimitedVec<crate::parse::EnumVariant>,
        enum_name: &str,
    ) -> TokenStream {
        let mut output = TokenStream::new();

        for (idx, variant_delimited) in variants_cdv.0.iter().enumerate() {
            let variant = &variant_delimited.value;
            let variant_name = variant.name.to_string();

            let mut variant_tokens = TokenStream::new();
            quote::ToTokens::to_tokens(variant, &mut variant_tokens);

            let documented =
                self.inject_doc_for_enum_variant(variant_tokens, enum_name, &variant_name);
            output.extend(documented);

            if idx < variants_cdv.0.len() - 1 {
                output.extend(quote::quote! { , });
            }
        }

        output
    }

    fn inject_doc_for_enum_variant(
        &self,
        variant_tokens: TokenStream,
        enum_name: &str,
        variant_name: &str,
    ) -> TokenStream {
        let mut path_parts = vec![self.base_path.clone()];
        path_parts.extend(self.context.iter().cloned());
        path_parts.push(format!("{}/{}.md", enum_name, variant_name));

        let full_path = path_parts.join("/");
        inject_doc_attr(full_path, self.cfg_attr.clone(), variant_tokens)
    }

    fn inject_doc_into_item(&self, func_tokens: TokenStream, fn_name: &str) -> TokenStream {
        let mut path_parts = vec![self.base_path.clone()];
        path_parts.extend(self.context.iter().cloned());
        path_parts.push(format!("{}.md", fn_name));

        let full_path = path_parts.join("/");
        let args = quote::quote! { path = #full_path };

        match syncdoc_impl(args, func_tokens.clone()) {
            Ok(instrumented) => instrumented,
            Err(e) => {
                eprintln!("syncdoc_impl failed: {}", e);
                func_tokens
            }
        }
    }

    fn inject_doc_into_simple_item(
        &self,
        item_tokens: TokenStream,
        item_name: &str,
    ) -> TokenStream {
        let mut path_parts = vec![self.base_path.clone()];
        path_parts.extend(self.context.iter().cloned());
        path_parts.push(format!("{}.md", item_name));

        let full_path = path_parts.join("/");
        inject_doc_attr(full_path, self.cfg_attr.clone(), item_tokens)
    }
}

fn extract_type_name(
    target_type: &unsynn::Many<
        unsynn::Cons<
            unsynn::Except<unsynn::Either<crate::parse::KFor, unsynn::BraceGroup>>,
            proc_macro2::TokenTree,
        >,
    >,
) -> String {
    if let Some(first) = target_type.0.first() {
        if let proc_macro2::TokenTree::Ident(ident) = &first.value.second {
            return ident.to_string();
        }
    }
    "Unknown".to_string()
}

fn extract_first_ident_from_tokens(
    tokens: &unsynn::Many<unsynn::Cons<unsynn::Except<unsynn::BraceGroup>, proc_macro2::TokenTree>>,
) -> String {
    if let Some(first) = tokens.0.first() {
        if let proc_macro2::TokenTree::Ident(ident) = &first.value.second {
            return ident.to_string();
        }
    }
    "Unknown".to_string()
}
