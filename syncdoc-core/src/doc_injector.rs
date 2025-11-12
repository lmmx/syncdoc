//! Procedural macro attributes for automatically injecting documentation from files.

use proc_macro2::TokenStream;
use quote::quote;
use unsynn::*;

use crate::parse::{SyncDocArg, SyncDocInner, FnSig};

pub fn syncdoc_impl(
    args: TokenStream,
    item: TokenStream,
) -> core::result::Result<TokenStream, TokenStream> {
    // Parse the syncdoc arguments
    let syncdoc_args = match parse_syncdoc_args(&mut args.to_token_iter()) {
        Ok(args) => args,
        Err(e) => {
            // Return both error and original item to preserve valid syntax
            return Ok(quote! {
                compile_error!(#e);
                #item
            });
        }
    };

    // Parse the function
    let mut item_iter = item.to_token_iter();
    let func = match parse_simple_function(&mut item_iter) {
        Ok(func) => func,
        Err(e) => {
            return Ok(quote! {
                compile_error!(#e);
                #item
            });
        }
    };

    Ok(generate_documented_function(syncdoc_args, func))
}

#[derive(Debug)]
struct SyncDocArgs {
    base_path: String,
    name: Option<String>,
    cfg_attr: Option<String>,
}

struct SimpleFunction {
    attrs: Vec<TokenStream>,
    vis: Option<TokenStream>,
    const_kw: Option<TokenStream>,
    async_kw: Option<TokenStream>,
    unsafe_kw: Option<TokenStream>,
    extern_kw: Option<TokenStream>,
    fn_name: proc_macro2::Ident,
    generics: Option<TokenStream>,
    params: TokenStream,
    ret_type: Option<TokenStream>,
    where_clause: Option<TokenStream>,
    body: TokenStream,
}

fn parse_syncdoc_args(input: &mut TokenIter) -> core::result::Result<SyncDocArgs, String> {
    match input.parse::<SyncDocInner>() {
        Ok(parsed) => {
            let mut args = SyncDocArgs {
                base_path: String::new(),
                name: None,
                cfg_attr: None,
            };

            if let Some(arg_list) = parsed.args {
                for arg in arg_list.0 {
                    match arg.value {
                        SyncDocArg::Path(path_arg) => {
                            args.base_path = path_arg.value.as_str().to_string();
                        }
                        SyncDocArg::Name(name_arg) => {
                            args.name = Some(name_arg.value.as_str().to_string());
                        }
                        SyncDocArg::CfgAttr(cfg_attr_arg) => {
                            args.cfg_attr = Some(cfg_attr_arg.value.as_str().to_string());
                        }
                    }
                }
            }

            if args.base_path.is_empty() || args.cfg_attr.is_none() {
                // Get the call site's file path if there might be config we could use there
                let call_site = proc_macro2::Span::call_site();
                let source_file = call_site.local_file()
                    .ok_or("Could not determine source file location")?
                    .to_string_lossy()
                    .to_string();

                // If macro path and TOML docs-path both unset, we don't know where to find the docs
                if args.base_path.is_empty() {
                    args.base_path = crate::config::get_docs_path(&source_file)
                        .map_err(|e| format!("Failed to get docs path from config: {}", e))?;
                }

                // We don't error on unconfigured cfg_attr, it's optional
                if let Ok(cfg) = crate::config::get_cfg_attr(&source_file) {
                    args.cfg_attr = cfg;
                }
            }

            Ok(args)
        }
        Err(e) => Err(format!("Failed to parse syncdoc args: {}", e)),
    }
}

fn parse_simple_function(input: &mut TokenIter) -> core::result::Result<SimpleFunction, String> {
    match input.parse::<FnSig>() {
        Ok(parsed) => {
            // Handle attributes
            let attrs = if let Some(attr_list) = parsed.attributes {
                attr_list
                    .0
                    .into_iter()
                    .map(|attr| {
                        let mut tokens = TokenStream::new();
                        unsynn::ToTokens::to_tokens(&attr, &mut tokens);
                        tokens
                    })
                    .collect()
            } else {
                Vec::new()
            };

            // Handle visibility
            let vis = parsed.visibility.map(|v| {
                let mut tokens = TokenStream::new();
                quote::ToTokens::to_tokens(&v, &mut tokens);
                tokens
            });

            // Handle const keyword
            let const_kw = parsed.const_kw.map(|k| {
                let mut tokens = TokenStream::new();
                unsynn::ToTokens::to_tokens(&k, &mut tokens);
                tokens
            });

            // Handle async keyword
            let async_kw = parsed.async_kw.map(|k| {
                let mut tokens = TokenStream::new();
                unsynn::ToTokens::to_tokens(&k, &mut tokens);
                tokens
            });

            // Handle unsafe keyword
            let unsafe_kw = parsed.unsafe_kw.map(|k| {
                let mut tokens = TokenStream::new();
                unsynn::ToTokens::to_tokens(&k, &mut tokens);
                tokens
            });

            // Handle extern keyword
            let extern_kw = parsed.extern_kw.map(|k| {
                let mut tokens = TokenStream::new();
                unsynn::ToTokens::to_tokens(&k, &mut tokens);
                tokens
            });

            let fn_name = parsed.name;

            let generics = parsed.generics.map(|g| {
                let mut tokens = TokenStream::new();
                unsynn::ToTokens::to_tokens(&g, &mut tokens);
                tokens
            });

            let mut params = TokenStream::new();
            unsynn::ToTokens::to_tokens(&parsed.params, &mut params);

            let ret_type = parsed.return_type.map(|rt| {
                let mut tokens = TokenStream::new();
                unsynn::ToTokens::to_tokens(&rt, &mut tokens);
                tokens
            });

            let where_clause = parsed.where_clause.map(|wc| {
                let mut tokens = TokenStream::new();
                unsynn::ToTokens::to_tokens(&wc, &mut tokens);
                tokens
            });

            let mut body = TokenStream::new();
            unsynn::ToTokens::to_tokens(&parsed.body, &mut body);

            Ok(SimpleFunction {
                attrs,
                vis,
                const_kw,
                async_kw,
                unsafe_kw,
                extern_kw,
                fn_name,
                generics,
                params,
                ret_type,
                where_clause,
                body,
            })
        }
        Err(e) => Err(format!("Failed to parse function: {}", e)),
    }
}

fn generate_documented_function(args: SyncDocArgs, func: SimpleFunction) -> TokenStream {
    let SimpleFunction {
        attrs,
        vis,
        const_kw,
        async_kw,
        unsafe_kw,
        extern_kw,
        fn_name,
        generics,
        params,
        ret_type,
        where_clause,
        body,
    } = func;

    // Construct the doc path
    let doc_file_name = args.name.unwrap_or_else(|| fn_name.to_string());
    let doc_path = if args.base_path.ends_with(".md") {
        // Direct file path provided
        args.base_path
    } else {
        // Directory path provided, append function name
        format!("{}/{}.md", args.base_path, doc_file_name)
    };

    // Generate tokens for all the modifiers
    let vis_tokens = vis.unwrap_or_default();
    let const_tokens = const_kw.unwrap_or_default();
    let async_tokens = async_kw.unwrap_or_default();
    let unsafe_tokens = unsafe_kw.unwrap_or_default();
    let extern_tokens = extern_kw.unwrap_or_default();
    let generics_tokens = generics.unwrap_or_default();
    let ret_tokens = ret_type.unwrap_or_default();
    let where_tokens = where_clause.unwrap_or_default();

    // Generate the documented function
    let doc_attr = if let Some(cfg_value) = args.cfg_attr {
        let cfg_ident = proc_macro2::Ident::new(&cfg_value, proc_macro2::Span::call_site());
        quote! { #[cfg_attr(#cfg_ident, doc = include_str!(#doc_path))] }
    } else {
        quote! { #[doc = include_str!(#doc_path)] }
    };

    quote! {
        #(#attrs)*
        #doc_attr
        #vis_tokens #const_tokens #async_tokens #unsafe_tokens #extern_tokens fn #fn_name #generics_tokens #params #ret_tokens #where_tokens #body
    }
}

/// Injects a doc attribute without parsing the item structure
pub fn inject_doc_attr(doc_path: String, item: TokenStream) -> TokenStream {
    if let Some(cfg_value) = cfg_attr {
        let cfg_ident = proc_macro2::Ident::new(&cfg_value, proc_macro2::Span::call_site());
        quote! {
            #[cfg_attr(#cfg_ident, doc = include_str!(#doc_path))]
            #item
        }
    } else {
        quote! {
            #[doc = include_str!(#doc_path)]
            #item
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_format::{Formatter, RustFmt};

    fn format_and_print(tokens: proc_macro2::TokenStream) -> String {
        let fmt_str = RustFmt::default()
            .format_tokens(tokens)
            .unwrap_or_else(|e| panic!("Format error: {}", e));
        println!("Generated code: {}", fmt_str);
        fmt_str
    }

    #[test]
    fn test_basic_doc_injection() {
        let args = quote!(path = "../docs");
        let item = quote! {
            fn test_function(x: u32) -> u32 {
                x + 1
            }
        };

        let result = syncdoc_impl(args, item);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = format_and_print(output);

        assert!(output_str.replace(" ", "").contains("include_str!"));
        assert!(output_str.contains("../docs/test_function.md"));
        assert!(output_str.contains("fn test_function"));
    }

    #[test]
    fn test_custom_name() {
        let args = quote!(path = "../docs", name = "custom");
        let item = quote! {
            fn test_function() {}
        };

        let result = syncdoc_impl(args, item);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = format_and_print(output);

        assert!(output_str.contains("../docs/custom.md"));
    }

    #[test]
    fn test_async_function_doc() {
        let args = quote!(path = "../docs");
        let item = quote! {
            async fn test_async() {
                println!("async test");
            }
        };

        let result = syncdoc_impl(args, item);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = format_and_print(output);

        assert!(output_str.contains("async fn test_async"));
        assert!(output_str.replace(" ", "").contains("include_str!"));
    }

    #[test]
    fn test_unsafe_function_doc() {
        let args = quote!(path = "../docs");
        let item = quote! {
            unsafe fn test_unsafe() {
                println!("unsafe test");
            }
        };

        let result = syncdoc_impl(args, item);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = format_and_print(output);

        assert!(output_str.contains("unsafe fn test_unsafe"));
        assert!(output_str.replace(" ", "").contains("include_str!"));
    }

    #[test]
    fn test_pub_async_function_doc() {
        let args = quote!(path = "../docs");
        let item = quote! {
            pub async fn test_pub_async() {
                println!("pub async test");
            }
        };

        let result = syncdoc_impl(args, item);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = format_and_print(output);

        assert!(output_str.contains("pub async fn test_pub_async"));
        assert!(output_str.replace(" ", "").contains("include_str!"));
    }

    #[test]
    fn test_direct_file_path() {
        let args = quote!(path = "../docs/special.md");
        let item = quote! {
            fn test_function() {}
        };

        let result = syncdoc_impl(args, item);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = format_and_print(output);

        assert!(output_str.contains("../docs/special.md"));
        assert!(!output_str.contains("test_function.md"));
    }
}
