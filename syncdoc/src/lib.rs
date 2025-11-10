#![warn(missing_docs)]
#![warn(clippy::std_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![forbid(unsafe_code)]

//! # syncdoc
//!
//! A procedural macro crate for injecting documentation from external files.
//!
//! This crate provides macros to automatically add documentation attributes to your functions
//! by reading from external markdown files.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

/// Injects documentation from an external file into a function.
///
/// # Examples
///
/// Document a single function:
/// ```ignore
/// # use syncdoc::syncdoc;
/// #[syncdoc(path = "../docs/my_function.md")]
/// pub fn my_function(my_arg: usize) {
///     println!("inside my_function!");
/// }
/// ```
///
/// With a custom name:
/// ```ignore
/// # use syncdoc::syncdoc;
/// #[syncdoc(path = "../docs", name = "custom_name")]
/// pub fn my_function() {
///     // Will look for ../docs/custom_name.md
/// }
/// ```
#[proc_macro_attribute]
pub fn syncdoc(args: TokenStream, item: TokenStream) -> TokenStream {
    let args2: TokenStream2 = args.into();
    let item2: TokenStream2 = item.into();

    match syncdoc_core::syncdoc_impl(args2, item2) {
        Ok(tokens) => tokens.into(),
        Err(error_tokens) => error_tokens.into(),
    }
}

/// Injects documentation from external files for all functions within a module or impl block.
///
/// This macro applies documentation injection to every function found within
/// the annotated module or impl block, automatically reading documentation from
/// external markdown files based on a hierarchical path structure.
///
/// # Examples
///
/// Document all functions in a module:
/// ```ignore
/// # use syncdoc::omnidoc;
/// #[omnidoc(path = "../docs")]
/// mod my_module {
///     pub fn function_one(x: i32) {
///         // Docs from ../docs/my_module/function_one.md
///         println!("Function one called with {}", x);
///     }
///     
///     pub fn function_two() {
///         // Docs from ../docs/my_module/function_two.md
///         println!("Function two called");
///     }
/// }
/// ```
///
/// Document all methods in an impl block:
/// ```ignore
/// # use syncdoc::omnidoc;
/// struct MyStruct;
///
/// #[omnidoc(path = "../docs")]
/// impl MyStruct {
///     pub fn method_one(&self, value: String) {
///         // Docs from ../docs/MyStruct/method_one.md
///         println!("Method called with {}", value);
///     }
///     
///     pub fn method_two(&self) {
///         // Docs from ../docs/MyStruct/method_two.md
///         println!("Another method called");
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn omnidoc(args: TokenStream, input: TokenStream) -> TokenStream {
    let args2: TokenStream2 = args.into();
    let input2: TokenStream2 = input.into();
    
    match syncdoc_core::inject_all_docs_impl(args2, input2) {
        Ok(output) => output.into(),
        Err(error_tokens) => error_tokens.into(),
    }
}