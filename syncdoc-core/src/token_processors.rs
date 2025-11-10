use crate::syncdoc_impl;
use proc_macro2::TokenStream;
use unsynn::*;

use crate::parse::{ImplBlockSig, ModuleContent, ModuleItem, ModuleSig, TraitSig};

pub(crate) struct TokenProcessor {
    input: TokenStream,
    base_path: String,
    context: Vec<String>,
}

impl TokenProcessor {
    pub(crate) fn new(input: TokenStream, base_path: String) -> Self {
        Self {
            input,
            base_path,
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

        let content = match self.input.clone().into_token_iter().parse::<ModuleContent>() {
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
                self.inject_doc_into_function(func_tokens, &func_sig.name.to_string())
            }
            ModuleItem::ImplBlock(impl_block) => self.process_impl_block(impl_block),
            ModuleItem::Module(module) => self.process_module_block(module),
            ModuleItem::Trait(trait_def) => self.process_trait_block(trait_def),
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

    fn wrap_in_braces(&self, content: TokenStream) -> TokenStream {
        let mut output = TokenStream::new();
        let group = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, content);
        output.extend(std::iter::once(proc_macro2::TokenTree::Group(group)));
        output
    }

    fn inject_doc_into_function(&self, func_tokens: TokenStream, fn_name: &str) -> TokenStream {
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
}

fn extract_type_name(target_type: &unsynn::Many<unsynn::Cons<unsynn::Except<unsynn::Either<crate::parse::KFor, unsynn::BraceGroup>>, proc_macro2::TokenTree>>) -> String {
    // Extract just the type name from the target_type tokens
    // This is a simplified version - for complex cases we might need more sophistication
    if let Some(first) = target_type.0.first() {
        if let proc_macro2::TokenTree::Ident(ident) = &first.value.second {
            return ident.to_string();
        }
    }
    "Unknown".to_string()
}

#[cfg(test)]
mod tests;
