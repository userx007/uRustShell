//! # Shortcut Dispatcher Macro
//!
//! This procedural macro generates a `no_std`-compatible command dispatcher module
//! based on a compact shortcut mapping file. It is designed for embedded or constrained
//! environments where heap allocation is limited or unavailable.
//!
//! ## Purpose
//! - Parses a shortcut mapping file at compile time.
//! - Registers shortcut keys mapped to function paths.
//! - Provides a dispatcher function that matches input strings to registered shortcuts
//!   and invokes the corresponding function.
//! - Includes helper functions to list all available shortcuts and check if a shortcut is supported.
//!
//! ## Macro Input Format
//!
//! ```rust
//! mod <module_name>;
//! shortcut_size = <expression>;
//! path = "<file_path>";
//! ```
//!
//! - `mod <module_name>`: Name of the generated module.
//! - `shortcut_size`: Maximum size of the shortcut string buffer (used in error reporting).
//! - `path`: Path to the file containing shortcut mappings.
//!
//! ## Generated API
//! - `dispatch(input: &str) -> Result<(), heapless::String<N>>`
//! - `is_supported_shortcut(input: &str) -> bool`
//! - `get_shortcuts() -> &'static str`

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, LitStr, Token,
};


/// Struct to parse macro input in the format:
/// `mod <name>; shortcut_size = <expr>; path = "<file_path>"`
struct ShortcutMacroInput {
    _mod_token: Token![mod],        // Token for the `mod` keyword
    mod_name: Ident,                // Identifier for the module name
    _semi1: Token![;],              // Semicolon after module declaration
    _shortcut_size_token: Ident,    // Identifier for `shortcut_size` keyword
    _eq_token: Token![=],           // Equals sign for shortcut_size assignment
    shortcut_size: Expr,            // Expression representing the shortcut size
    _semi2: Token![;],              // Semicolon after shortcut_size declaration
    _path_token: Ident,             // Identifier for `path` keyword
    _eq_token2: Token![=],          // Equals sign for path assignment
    path: LitStr,                   // Literal string representing the file path
}


impl Parse for ShortcutMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ShortcutMacroInput {
            _mod_token: input.parse()?,
            mod_name: input.parse()?,
            _semi1: input.parse()?,
            _shortcut_size_token: input.parse()?,
            _eq_token: input.parse()?,
            shortcut_size: input.parse()?,
            _semi2: input.parse()?,
            _path_token: input.parse()?,
            _eq_token2: input.parse()?,
            path: input.parse()?,
        })
    }
}


pub fn define_shortcuts_impl(input: TokenStream) -> TokenStream {
    let ShortcutMacroInput {
        mod_name,
        shortcut_size,
        path,
        ..
    } = parse_macro_input!(input as ShortcutMacroInput);

    // Resolve path relative to the crate invoking the macro
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let full_path = std::path::Path::new(&manifest_dir).join(path.value());

    let raw = std::fs::read_to_string(&full_path)
        .expect(&format!("Failed to read shortcut file: {:?}", full_path));

    let mut match_arms = vec![];
    let mut prefixes = std::collections::HashSet::new();
    let mut shortcut_keys = vec![];
    let mut buffer = String::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        buffer.push_str(line);
        if line.ends_with("},") {
            if let Some((prefix, rest)) = buffer.split_once(':') {
                let prefix = prefix.trim();
                prefixes.insert(prefix.to_string());

                for entry in rest.split(',') {
                    let entry = entry.trim().trim_matches('{').trim_matches('}').trim();
                    if entry.is_empty() {
                        continue;
                    }
                    if let Some((key, func)) = entry.split_once(':') {
                        let key = key.trim();
                        let func = func.trim();
                        if let Ok(path) = syn::parse_str::<syn::Path>(func) {
                            let full_key = format!("{}{}", prefix, key);
                            shortcut_keys.push(full_key.clone());
                            match_arms.push(quote! {
                                #full_key => {
                                    #path(param);
                                    Ok(())
                                },
                            });
                        } else {
                            panic!("Invalid function path: {}", func);
                        }
                    }
                }
            }
            buffer.clear();
        }
    }

    let supported_checks = prefixes.iter().map(|p| {
        quote! { c == #p }
    });

    let shortcut_list = shortcut_keys.join(" | ");
    let list_fn = quote! {
        pub fn get_shortcuts() -> &'static str {
            #shortcut_list
        }
    };

    let support_fn = quote! {
        pub fn is_supported_shortcut(input: &str) -> bool {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                return false;
            }
            let c = &trimmed[0..1];
            #( #supported_checks )||*
        }
    };

    let dispatch_fn = quote! {
        pub fn dispatch(input: &str) -> Result<(), heapless::String<{ #shortcut_size }>> {
            let trimmed = input.trim();
            let (key, param) = if trimmed.len() >= 2 {
                let key = &trimmed[..2];
                let param = trimmed[2..].trim();
                (key, param)
            } else {
                (trimmed, "")
            };
            match key {
                #( #match_arms )*
                _ => {
                    let mut msg = heapless::String::<{#shortcut_size}>::new();
                    use core::fmt::Write;
                    let _ = write!(msg, "Unknown shortcut: {}", key);
                    Err(msg)
                },
            }
        }
    };

    let expanded = quote! {
        #[cfg_attr(not(test), no_std)]
        use core::fmt::Write;
        pub mod #mod_name {
            #dispatch_fn
            #support_fn
            #list_fn
        }
    };

    TokenStream::from(expanded)
}