//! Procedural macro attributes for automatically injecting documentation from files.

use proc_macro2::TokenStream;
use quote::quote;
use unsynn::*;

use crate::parse::{SyncDocArg, SyncDocInner};
use crate::path_utils::make_manifest_relative_path;

/// Injects a doc attribute without parsing the item structure
pub fn omnidoc_impl(doc_path: String, cfg_attr: Option<String>, item: TokenStream) -> TokenStream {
    // Get the call site's file path if there might be config we could use there
    let call_site = proc_macro2::Span::call_site();
    let local_file = call_site.local_file().expect("Could not find local file");
    let rel_doc_path = make_manifest_relative_path(&doc_path, &local_file);
    if let Some(cfg_value) = cfg_attr {
        let cfg_ident = proc_macro2::Ident::new(&cfg_value, proc_macro2::Span::call_site());
        quote! {
            #[cfg_attr(#cfg_ident, doc = include_str!(#rel_doc_path))]
            #item
        }
    } else {
        quote! {
            #[doc = include_str!(#rel_doc_path)]
            #item
        }
    }
}

/// Implementation for the module_doc!() macro
///
/// Generates an include_str!() call with the automatically resolved path
/// to the module's markdown documentation file.
pub fn module_doc_impl(args: TokenStream) -> core::result::Result<TokenStream, TokenStream> {
    let call_site = proc_macro2::Span::call_site();
    let source_file = call_site
        .local_file()
        .ok_or_else(|| {
            let error = "Could not determine source file location";
            quote! { compile_error!(#error) }
        })?
        .to_string_lossy()
        .to_string();

    // Parse the arguments to get base_path if provided
    let base_path = if args.is_empty() {
        // No args provided, get from config
        crate::config::get_docs_path(&source_file).map_err(|e| {
            let error = format!("Failed to get docs path from config: {}", e);
            quote! { compile_error!(#error) }
        })?
    } else {
        // Parse args to extract path
        let mut args_iter = args.into_token_iter();
        match parse_syncdoc_args(&mut args_iter) {
            Ok(parsed_args) => parsed_args.base_path,
            Err(e) => {
                let error = format!("Failed to parse module_doc args: {}", e);
                return Err(quote! { compile_error!(#error) });
            }
        }
    };

    // Extract module path and construct full doc path
    let module_path = crate::path_utils::extract_module_path(&source_file);
    let doc_path = if module_path.is_empty() {
        // For lib.rs or main.rs, use the file stem
        let file_stem = std::path::Path::new(&source_file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("module");
        format!("{}/{}.md", base_path, file_stem)
    } else {
        format!("{}/{}.md", base_path, module_path)
    };

    // Make path relative to call site
    let local_file = call_site.local_file().ok_or_else(|| {
        let error = "Could not find local file";
        quote! { compile_error!(#error) }
    })?;
    let rel_doc_path = make_manifest_relative_path(&doc_path, &local_file);

    // Generate include_str!() call
    Ok(quote! {
        include_str!(#rel_doc_path)
    })
}

#[derive(Debug)]
struct SyncDocArgs {
    base_path: String,
    name: Option<String>,
    cfg_attr: Option<String>,
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
                // If macro path and TOML docs-path both unset, we don't know where to find the docs
                if args.base_path.is_empty() {
                    // Get the call site's file path if there might be config we could use there
                    let call_site = proc_macro2::Span::call_site();
                    let source_file = call_site
                        .local_file()
                        .ok_or("Could not determine source file location")?
                        .to_string_lossy()
                        .to_string();

                    let base_path = crate::config::get_docs_path(&source_file)
                        .map_err(|e| format!("Failed to get docs path from config: {}", e))?;

                    // Extract module path and prepend to base_path
                    let module_path = crate::path_utils::extract_module_path(&source_file);
                    args.base_path = if module_path.is_empty() {
                        base_path
                    } else {
                        format!("{}/{}", base_path, module_path)
                    };
                }

                // We don't error on unconfigured cfg_attr, it's optional
                if args.cfg_attr.is_none() {
                    let call_site = proc_macro2::Span::call_site();
                    if let Some(source_path) = call_site.local_file() {
                        let source_file = source_path.to_string_lossy().to_string();
                        if let Ok(cfg) = crate::config::get_cfg_attr(&source_file) {
                            args.cfg_attr = cfg;
                        }
                    }
                }
            }

            Ok(args)
        }
        Err(e) => Err(format!("Failed to parse syncdoc args: {}", e)),
    }
}
