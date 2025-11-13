use proc_macro2::TokenStream;
use quote::quote;
use std::path::Path;
use unsynn::*;

use crate::config::{get_cfg_attr, get_docs_path};
use crate::parse::{SyncDocArg, SyncDocInner};
use crate::path_utils::apply_module_path;
use crate::token_processors::TokenProcessor;

pub fn inject_all_docs_impl(
    args: TokenStream,
    input: TokenStream,
) -> core::result::Result<TokenStream, TokenStream> {
    let (base_path, cfg_attr) = match parse_path_from_args(args) {
        Ok(result) => result,
        Err(e) => {
            let error_msg = e.to_string();
            return Ok(quote! {
                compile_error!(#error_msg);
                #input
            });
        }
    };

    Ok(TokenProcessor::new(input, base_path, cfg_attr).process())
}

fn parse_path_from_args(
    args: TokenStream,
) -> core::result::Result<(String, Option<String>), String> {
    // If no args provided, try to get from config
    if args.is_empty() {
        let call_site = proc_macro2::Span::call_site();
        if let Some(source_path) = call_site.local_file() {
            let source_file = source_path.to_string_lossy().to_string();
            let base_path = get_docs_path(Path::new(&source_file))
                .map_err(|e| format!("Failed to get docs path from config: {}", e))?;
            let cfg_attr = get_cfg_attr().ok().flatten();

            let path = apply_module_path(base_path);

            return Ok((path, cfg_attr));
        } else {
            return Err("omnidoc requires a path argument".to_string());
        }
    }

    let mut args_iter = args.into_token_iter();
    match args_iter.parse::<SyncDocInner>() {
        Ok(parsed) => {
            let mut path = None;
            let mut cfg_attr = None;

            if let Some(arg_list) = parsed.args {
                for arg in arg_list.0 {
                    match arg.value {
                        SyncDocArg::Path(path_arg) => {
                            path = Some(path_arg.value.as_str().to_string());
                        }
                        SyncDocArg::CfgAttr(cfg_arg) => {
                            cfg_attr = Some(cfg_arg.value.as_str().to_string());
                        }
                        _ => {}
                    }
                }
            }

            let path = if let Some(p) = path {
                apply_module_path(p)
            } else {
                // Try config
                let call_site = proc_macro2::Span::call_site();
                if let Some(source_path) = call_site.local_file() {
                    let source_file = source_path.to_string_lossy().to_string();
                    let base_path = get_docs_path(Path::new(&source_file))
                        .map_err(|e| format!("Failed to get docs path from config: {}", e))?;
                    apply_module_path(base_path)
                } else {
                    return Err("path argument not found".to_string());
                }
            };

            // If cfg_attr still None, try config
            if cfg_attr.is_none() {
                cfg_attr = get_cfg_attr().ok().flatten();
            }

            Ok((path, cfg_attr))
        }
        Err(_e) => Err("Failed to parse arguments".to_string()),
    }
}
