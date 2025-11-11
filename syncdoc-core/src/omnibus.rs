use proc_macro2::TokenStream;
use quote::quote;
use unsynn::*;

use crate::parse::{DocStubArg, DocStubInner};
use crate::token_processors::TokenProcessor;

pub fn inject_all_docs_impl(args: TokenStream, input: TokenStream) -> core::result::Result<TokenStream, TokenStream> {
    // Parse the path argument
    let base_path = match parse_path_from_args(args) {
        Ok(path) => path,
        Err(e) => {
            let error_msg = e.to_string();
            // Return both the error and the original input to avoid syntax errors
            return Ok(quote! {
                compile_error!(#error_msg);
                #input
            });
        }
    };

    // Process the input with the base path
    Ok(TokenProcessor::new(input, base_path).process())
}

fn parse_path_from_args(args: TokenStream) -> core::result::Result<String, String> {
    // If no args provided, try to get from config (only if span info available)
    if args.is_empty() {
        let call_site = proc_macro2::Span::call_site();
        if let Some(source_path) = call_site.local_file() {
            let source_file = source_path.to_string_lossy().to_string();
            return crate::config::get_docs_path(&source_file)
                .map_err(|e| format!("Failed to get docs path from config: {}", e));
        } else {
            return Err("omnidoc requires a path argument".to_string());
        }
    }

    let mut args_iter = args.into_token_iter();
    match args_iter.parse::<DocStubInner>() {
        Ok(parsed) => {
            if let Some(arg_list) = parsed.args {
                for arg in arg_list.0 {
                    if let DocStubArg::Path(path_arg) = arg.value {
                        return Ok(path_arg.value.as_str().to_string());
                    }
                }
            }
            // If parsed but no path found, try config (only if span info available)
            let call_site = proc_macro2::Span::call_site();
            if let Some(source_path) = call_site.local_file() {
                let source_file = source_path.to_string_lossy().to_string();
                crate::config::get_docs_path(&source_file)
                    .map_err(|e| format!("Failed to get docs path from config: {}", e))
            } else {
                Err("path argument not found".to_string())
            }
        }
        Err(_e) => Err("Failed to parse arguments".to_string()),
    }
}
