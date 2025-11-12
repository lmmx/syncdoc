use crate::{inject_doc_attr, syncdoc_impl};
use proc_macro2::TokenStream;
use unsynn::*;

use crate::parse::{ImplBlockSig, ModuleContent, ModuleItem, ModuleSig, TraitSig};

pub(crate) struct TokenProcessor {
    input: TokenStream,
    base_path: String,
    cfg_attr: Option<String>,
    context: Vec<String>,
}

impl TokenProcessor {
    pub(crate) fn new(input: TokenStream, base_path: String, cfg_attr: Option<String>) -> Self {
        Self {
            input,
            base_path,
            cfg_attr,
            context: Vec::new(),
        }
    }

    pub(crate) fn process(self) -> TokenStream {
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
        // Extract the struct/type name for context
        let type_name = extract_type_name(&impl_block.target_type);

        // Get the body content as TokenStream
        let body_stream = {
            let mut ts = TokenStream::new();
            impl_block.body.to_tokens(&mut ts);
            // Extract content from within braces
            if let Some(proc_macro2::TokenTree::Group(group)) = ts.into_iter().next() {
                group.stream()
            } else {
                TokenStream::new()
            }
        };

        // Create new processor with updated context
        let mut new_context = self.context.clone();
        new_context.push(type_name);
        let new_processor = TokenProcessor {
            input: body_stream,
            base_path: self.base_path.clone(),
            context: new_context,
        };

        let processed_content = new_processor.process();
        let processed_body = self.wrap_in_braces(processed_content);

        // Reconstruct the impl block with processed body
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

        output.extend(processed_body);

        output
    }

    fn process_module_block(&self, module: ModuleSig) -> TokenStream {
        // Get the body content as TokenStream
        let body_stream = {
            let mut ts = TokenStream::new();
            module.body.to_tokens(&mut ts);
            // Extract content from within braces
            if let Some(proc_macro2::TokenTree::Group(group)) = ts.into_iter().next() {
                group.stream()
            } else {
                TokenStream::new()
            }
        };

        // Create new processor with updated context
        let mut new_context = self.context.clone();
        new_context.push(module.name.to_string());
        let new_processor = TokenProcessor {
            input: body_stream,
            base_path: self.base_path.clone(),
            context: new_context,
        };

        let processed_content = new_processor.process();
        let processed_body = self.wrap_in_braces(processed_content);

        // Reconstruct the module with processed body
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

        output.extend(processed_body);

        output
    }

    fn process_trait_block(&self, trait_def: TraitSig) -> TokenStream {
        // Get the body content as TokenStream
        let body_stream = {
            let mut ts = TokenStream::new();
            trait_def.body.to_tokens(&mut ts);
            // Extract content from within braces
            if let Some(proc_macro2::TokenTree::Group(group)) = ts.into_iter().next() {
                group.stream()
            } else {
                TokenStream::new()
            }
        };

        // Create new processor with updated context (traits behave like modules)
        let mut new_context = self.context.clone();
        new_context.push(trait_def.name.to_string());
        let new_processor = TokenProcessor {
            input: body_stream,
            base_path: self.base_path.clone(),
            context: new_context,
        };

        let processed_content = new_processor.process();
        let processed_body = self.wrap_in_braces(processed_content);

        // Reconstruct the trait with processed body
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

        output.extend(processed_body);

        output
    }

    fn process_struct(&self, struct_sig: crate::parse::StructSig) -> TokenStream {
        let struct_name = struct_sig.name.to_string();

        // Process the struct body to add docs to fields
        let processed_body = match &struct_sig.body {
            crate::parse::StructBody::Named(brace_group) => {
                // Extract fields from brace group
                let body_stream = {
                    let mut ts = TokenStream::new();
                    brace_group.to_tokens(&mut ts);
                    // Extract content from within braces
                    if let Some(proc_macro2::TokenTree::Group(group)) = ts.into_iter().next() {
                        group.stream()
                    } else {
                        TokenStream::new()
                    }
                };

                let processed_fields = self.process_struct_fields(body_stream, &struct_name);
                self.wrap_in_braces(processed_fields)
            }
            crate::parse::StructBody::Tuple(tuple) => {
                // For tuple structs, process tuple fields
                let mut ts = TokenStream::new();
                tuple.to_tokens(&mut ts);
                ts
            }
            crate::parse::StructBody::Unit(semi) => {
                // Unit structs have no fields
                let mut ts = TokenStream::new();
                semi.to_tokens(&mut ts);
                ts
            }
        };

        // Reconstruct the struct with doc attribute
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

        let name_ident = struct_sig.name;
        name_ident.to_tokens(&mut output);

        if let Some(generics) = struct_sig.generics {
            generics.to_tokens(&mut output);
        }

        if let Some(where_clause) = struct_sig.where_clause {
            where_clause.to_tokens(&mut output);
        }

        // Inject doc for the struct itself
        let struct_with_doc = self.inject_doc_into_simple_item(output, &struct_name);

        // Combine struct declaration with processed body
        let mut final_output = struct_with_doc;
        final_output.extend(processed_body);

        final_output
    }

    fn process_struct_fields(&self, fields_stream: TokenStream, struct_name: &str) -> TokenStream {
        let mut output = TokenStream::new();
        let mut current_field = Vec::new();
        let mut depth = 0;

        for tt in fields_stream.into_iter() {
            match &tt {
                proc_macro2::TokenTree::Punct(punct) if punct.as_char() == ',' && depth == 0 => {
                    // End of field
                    if !current_field.is_empty() {
                        let field_tokens: TokenStream = current_field.drain(..).collect();
                        if let Some(field_name) = extract_struct_field_name(&field_tokens) {
                            let documented = self.inject_doc_for_struct_field(
                                field_tokens,
                                struct_name,
                                &field_name,
                            );
                            output.extend(documented);
                            output.extend(std::iter::once(tt));
                        } else {
                            output.extend(field_tokens);
                            output.extend(std::iter::once(tt));
                        }
                    }
                }
                proc_macro2::TokenTree::Group(g) => match g.delimiter() {
                    proc_macro2::Delimiter::Brace
                    | proc_macro2::Delimiter::Parenthesis
                    | proc_macro2::Delimiter::Bracket => {
                        depth += 1;
                        current_field.push(tt);
                    }
                    _ => current_field.push(tt),
                },
                _ => {
                    current_field.push(tt);
                }
            }
        }

        // Handle last field (no trailing comma)
        if !current_field.is_empty() {
            let field_tokens: TokenStream = current_field.drain(..).collect();
            if let Some(field_name) = extract_struct_field_name(&field_tokens) {
                let documented =
                    self.inject_doc_for_struct_field(field_tokens, struct_name, &field_name);
                output.extend(documented);
            } else {
                output.extend(field_tokens);
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

        // Use simpler injection for fields
        inject_doc_attr(full_path, field_tokens, self.cfg_attr.clone())
    }

    fn process_enum(&self, enum_sig: crate::parse::EnumSig) -> TokenStream {
        let enum_name = enum_sig.name.to_string();

        // Get the body content as TokenStream
        let body_stream = {
            let mut ts = TokenStream::new();
            enum_sig.body.to_tokens(&mut ts);
            // Extract content from within braces
            if let Some(proc_macro2::TokenTree::Group(group)) = ts.into_iter().next() {
                group.stream()
            } else {
                TokenStream::new()
            }
        };

        // Process enum variants
        let processed_variants = self.process_enum_variants(body_stream, &enum_name);
        let processed_body = self.wrap_in_braces(processed_variants);

        // Reconstruct the enum with doc attribute
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

        let name_ident = enum_sig.name;
        name_ident.to_tokens(&mut output);

        if let Some(generics) = enum_sig.generics {
            generics.to_tokens(&mut output);
        }

        if let Some(where_clause) = enum_sig.where_clause {
            where_clause.to_tokens(&mut output);
        }

        // Inject doc for the enum itself using simpler method
        let enum_with_doc = self.inject_doc_into_simple_item(output, &enum_name);

        // Combine enum declaration with processed body
        let mut final_output = enum_with_doc;
        final_output.extend(processed_body);

        final_output
    }

    fn process_enum_variants(&self, variants_stream: TokenStream, enum_name: &str) -> TokenStream {
        let mut output = TokenStream::new();
        let mut current_variant = Vec::new();
        let mut depth = 0;

        for tt in variants_stream.into_iter() {
            match &tt {
                proc_macro2::TokenTree::Punct(punct) if punct.as_char() == ',' && depth == 0 => {
                    // End of variant
                    if !current_variant.is_empty() {
                        let variant_tokens: TokenStream = current_variant.drain(..).collect();
                        if let Some(variant_name) = extract_first_ident(&variant_tokens) {
                            let documented = self.inject_doc_for_enum_variant(
                                variant_tokens,
                                enum_name,
                                &variant_name,
                            );
                            output.extend(documented);
                            output.extend(std::iter::once(tt));
                        } else {
                            output.extend(variant_tokens);
                            output.extend(std::iter::once(tt));
                        }
                    }
                }
                // proc_macro2::TokenTree::Group(_) => {
                //     current_variant.push(tt);
                // }
                proc_macro2::TokenTree::Group(g) => match g.delimiter() {
                    proc_macro2::Delimiter::Brace | proc_macro2::Delimiter::Parenthesis => {
                        depth += 1;
                        current_variant.push(tt);
                    }
                    _ => current_variant.push(tt),
                },
                _ => {
                    current_variant.push(tt);
                }
            }
        }

        // Handle last variant (no trailing comma)
        if !current_variant.is_empty() {
            let variant_tokens: TokenStream = current_variant.drain(..).collect();
            if let Some(variant_name) = extract_first_ident(&variant_tokens) {
                let documented =
                    self.inject_doc_for_enum_variant(variant_tokens, enum_name, &variant_name);
                output.extend(documented);
            } else {
                output.extend(variant_tokens);
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

        // Use simpler injection for variants
        inject_doc_attr(full_path, variant_tokens, self.cfg_attr.clone())
    }

    fn wrap_in_braces(&self, content: TokenStream) -> TokenStream {
        let mut output = TokenStream::new();
        let group = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, content);
        output.extend(std::iter::once(proc_macro2::TokenTree::Group(group)));
        output
    }

    fn inject_doc_into_item(&self, func_tokens: TokenStream, fn_name: &str) -> TokenStream {
        // Construct the full path including context
        let mut path_parts = vec![self.base_path.clone()];
        path_parts.extend(self.context.iter().cloned());
        path_parts.push(format!("{}.md", fn_name));

        let full_path = path_parts.join("/");

        // Create args token stream with the constructed path
        let args = quote::quote! { path = #full_path };

        match syncdoc_impl(args, func_tokens.clone()) {
            Ok(instrumented) => instrumented,
            Err(e) => {
                eprintln!("syncdoc_impl failed: {}", e);
                func_tokens // fallback to original
            }
        }
    }

    fn inject_doc_into_simple_item(
        &self,
        item_tokens: TokenStream,
        item_name: &str,
    ) -> TokenStream {
        // Construct the full path including context
        let mut path_parts = vec![self.base_path.clone()];
        path_parts.extend(self.context.iter().cloned());
        path_parts.push(format!("{}.md", item_name));

        let full_path = path_parts.join("/");

        // Use the simpler injection that doesn't parse
        inject_doc_attr(full_path, item_tokens, self.cfg_attr.clone())
    }
}

fn extract_struct_field_name(tokens: &TokenStream) -> Option<String> {
    let mut iter = tokens.clone().into_iter();

    // Skip attributes (#[...])
    while let Some(tt) = iter.next() {
        if let proc_macro2::TokenTree::Punct(punct) = &tt {
            if punct.as_char() == '#' {
                // Skip the attribute group
                if let Some(proc_macro2::TokenTree::Group(_)) = iter.next() {
                    continue;
                }
            }
        }

        // Skip visibility keywords (pub, pub(crate), etc.)
        if let proc_macro2::TokenTree::Ident(ident) = &tt {
            let s = ident.to_string();
            if s == "pub" {
                // Might be followed by (crate) or similar
                if let Some(proc_macro2::TokenTree::Group(_)) = iter.clone().next() {
                    iter.next();
                }
                continue;
            }
        }

        // First ident after visibility/attributes is the field name
        if let proc_macro2::TokenTree::Ident(ident) = tt {
            return Some(ident.to_string());
        }
    }

    None
}

fn extract_type_name(
    target_type: &unsynn::Many<
        unsynn::Cons<
            unsynn::Except<unsynn::Either<crate::parse::KFor, unsynn::BraceGroup>>,
            proc_macro2::TokenTree,
        >,
    >,
) -> String {
    // Extract just the type name from the target_type tokens
    // This is a simplified version - for complex cases we might need more sophistication
    if let Some(first) = target_type.0.first() {
        if let proc_macro2::TokenTree::Ident(ident) = &first.value.second {
            return ident.to_string();
        }
    }
    "Unknown".to_string()
}

fn extract_first_ident(tokens: &TokenStream) -> Option<String> {
    for tt in tokens.clone().into_iter() {
        if let proc_macro2::TokenTree::Ident(ident) = tt {
            return Some(ident.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests;
