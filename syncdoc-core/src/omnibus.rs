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
    // Get call site for relative path calculation
    let call_site = proc_macro2::Span::call_site();
    let source_file = call_site.local_file()
        .ok_or("Could not determine source file location")?
        .to_string_lossy()
        .to_string();

    // If no args provided, try to get from config
    if args.is_empty() {
        return crate::config::get_docs_path(&source_file)
            .map_err(|e| format!("Failed to get docs path from config: {}", e));
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
            // If parsed but no path found, try config
            crate::config::get_docs_path(&source_file)
                .map_err(|e| format!("Failed to get docs path from config: {}", e))
        }
        Err(_e) => Err("Failed to parse arguments".to_string()),
    }
}
