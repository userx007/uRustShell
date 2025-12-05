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
    Expr, Ident, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Struct to parse macro input in the format:
/// `mod <name>; shortcut_size = <expr>; path = "<file_path>"`
struct ShortcutMacroInput {
    _mod_token: Token![mod],     // Token for the `mod` keyword
    mod_name: Ident,             // Identifier for the module name
    _semi1: Token![;],           // Semicolon after module declaration
    _shortcut_size_token: Ident, // Identifier for `shortcut_size` keyword
    _eq_token: Token![=],        // Equals sign for shortcut_size assignment
    shortcut_size: Expr,         // Expression representing the shortcut size
    _semi2: Token![;],           // Semicolon after shortcut_size declaration
    _path_token: Ident,          // Identifier for `path` keyword
    _eq_token2: Token![=],       // Equals sign for path assignment
    path: LitStr,                // Literal string representing the file path
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

pub fn generate_shortcuts_dispatcher_from_file(input: TokenStream) -> TokenStream {
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
        .unwrap_or_else(|_| panic!("Failed to read shortcut file: {:?}", full_path));

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

// ================= TESTS ==========================

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Global state to track function calls in tests
    static CALL_LOG: Mutex<Option<HashMap<String, Vec<String>>>> = Mutex::new(None);

    fn ensure_log_initialized() {
        let mut log = CALL_LOG.lock().unwrap();
        if log.is_none() {
            *log = Some(HashMap::new());
        }
    }

    fn record_call(func_name: &str, param: &str) {
        ensure_log_initialized();
        let mut log = CALL_LOG.lock().unwrap();
        if let Some(ref mut map) = *log {
            map.entry(func_name.to_string())
                .or_insert_with(Vec::new)
                .push(param.to_string());
        }
    }

    fn get_calls(func_name: &str) -> Vec<String> {
        ensure_log_initialized();
        let log = CALL_LOG.lock().unwrap();
        log.as_ref()
            .and_then(|map| map.get(func_name).cloned())
            .unwrap_or_default()
    }

    fn clear_log() {
        ensure_log_initialized();
        let mut log = CALL_LOG.lock().unwrap();
        if let Some(ref mut map) = *log {
            map.clear();
        }
    }

    // Test command functions
    mod commands {
        use super::record_call;

        pub fn bang_plus(param: &str) {
            record_call("bang_plus", param);
        }

        pub fn bang_minus(param: &str) {
            record_call("bang_minus", param);
        }

        pub fn bang_hash(param: &str) {
            record_call("bang_hash", param);
        }

        pub fn plus_plus(param: &str) {
            record_call("plus_plus", param);
        }

        pub fn plus_minus(param: &str) {
            record_call("plus_minus", param);
        }

        pub fn plus_hash(param: &str) {
            record_call("plus_hash", param);
        }

        pub fn minus_plus(param: &str) {
            record_call("minus_plus", param);
        }

        pub fn minus_minus(param: &str) {
            record_call("minus_minus", param);
        }

        pub fn minus_hash(param: &str) {
            record_call("minus_hash", param);
        }

        pub fn hash_bang(param: &str) {
            record_call("hash_bang", param);
        }

        pub fn hash_plus(param: &str) {
            record_call("hash_plus", param);
        }

        pub fn hash_question(param: &str) {
            record_call("hash_question", param);
        }

        pub fn question_bang(param: &str) {
            record_call("question_bang", param);
        }

        pub fn question_plus(param: &str) {
            record_call("question_plus", param);
        }

        pub fn question_question(param: &str) {
            record_call("question_question", param);
        }
    }

    // Manual implementation for testing (simulating what the macro generates)
    mod shortcuts {
        use super::commands;

        pub fn dispatch(input: &str) -> Result<(), heapless::String<64>> {
            let trimmed = input.trim();
            let (key, param) = if trimmed.len() >= 2 {
                let key = &trimmed[..2];
                let param = trimmed[2..].trim();
                (key, param)
            } else {
                (trimmed, "")
            };

            match key {
                "!+" => {
                    commands::bang_plus(param);
                    Ok(())
                }
                "!-" => {
                    commands::bang_minus(param);
                    Ok(())
                }
                "!#" => {
                    commands::bang_hash(param);
                    Ok(())
                }
                "++" => {
                    commands::plus_plus(param);
                    Ok(())
                }
                "+-" => {
                    commands::plus_minus(param);
                    Ok(())
                }
                "+#" => {
                    commands::plus_hash(param);
                    Ok(())
                }
                "-+" => {
                    commands::minus_plus(param);
                    Ok(())
                }
                "--" => {
                    commands::minus_minus(param);
                    Ok(())
                }
                "-#" => {
                    commands::minus_hash(param);
                    Ok(())
                }
                "#!" => {
                    commands::hash_bang(param);
                    Ok(())
                }
                "#+" => {
                    commands::hash_plus(param);
                    Ok(())
                }
                "#?" => {
                    commands::hash_question(param);
                    Ok(())
                }
                "?!" => {
                    commands::question_bang(param);
                    Ok(())
                }
                "?+" => {
                    commands::question_plus(param);
                    Ok(())
                }
                "??" => {
                    commands::question_question(param);
                    Ok(())
                }
                _ => {
                    let mut msg = heapless::String::<64>::new();
                    use core::fmt::Write;
                    let _ = write!(msg, "Unknown shortcut: {}", key);
                    Err(msg)
                }
            }
        }

        pub fn is_supported_shortcut(input: &str) -> bool {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                return false;
            }
            let c = &trimmed[0..1];
            c == "!" || c == "+" || c == "-" || c == "#" || c == "?"
        }

        pub fn get_shortcuts() -> &'static str {
            "!+ | !- | !# | ++ | +- | +# | -+ | -- | -# | #! | #+ | #? | ?! | ?+ | ??"
        }
    }

    #[test]
    fn test_basic_dispatch() {
        clear_log();

        assert!(shortcuts::dispatch("!+").is_ok());
        assert_eq!(get_calls("bang_plus"), vec![""]);

        clear_log();
        assert!(shortcuts::dispatch("+-").is_ok());
        assert_eq!(get_calls("plus_minus"), vec![""]);

        clear_log();
        assert!(shortcuts::dispatch("#?").is_ok());
        assert_eq!(get_calls("hash_question"), vec![""]);
    }

    #[test]
    fn test_dispatch_with_parameters() {
        clear_log();

        assert!(shortcuts::dispatch("!+ hello world").is_ok());
        assert_eq!(get_calls("bang_plus"), vec!["hello world"]);

        clear_log();
        assert!(shortcuts::dispatch("-- 42").is_ok());
        assert_eq!(get_calls("minus_minus"), vec!["42"]);

        clear_log();
        assert!(shortcuts::dispatch("?? test param").is_ok());
        assert_eq!(get_calls("question_question"), vec!["test param"]);
    }

    #[test]
    fn test_dispatch_with_whitespace() {
        clear_log();

        // Leading whitespace
        assert!(shortcuts::dispatch("  !+").is_ok());
        assert_eq!(get_calls("bang_plus"), vec![""]);

        clear_log();
        // Trailing whitespace
        assert!(shortcuts::dispatch("+-  ").is_ok());
        assert_eq!(get_calls("plus_minus"), vec![""]);

        clear_log();
        // Both
        assert!(shortcuts::dispatch("  #!  ").is_ok());
        assert_eq!(get_calls("hash_bang"), vec![""]);

        clear_log();
        // Whitespace before parameter
        assert!(shortcuts::dispatch("?+   param").is_ok());
        assert_eq!(get_calls("question_plus"), vec!["param"]);
    }

    #[test]
    fn test_dispatch_unknown_shortcut() {
        let result = shortcuts::dispatch("@@");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown shortcut"));

        let result = shortcuts::dispatch("xy");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown shortcut"));
    }

    #[test]
    fn test_dispatch_partial_shortcut() {
        let result = shortcuts::dispatch("!");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown shortcut"));

        let result = shortcuts::dispatch("+");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown shortcut"));
    }

    #[test]
    fn test_dispatch_empty_input() {
        let result = shortcuts::dispatch("");
        assert!(result.is_err());

        let result = shortcuts::dispatch("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_supported_shortcut() {
        assert!(shortcuts::is_supported_shortcut("!"));
        assert!(shortcuts::is_supported_shortcut("+"));
        assert!(shortcuts::is_supported_shortcut("-"));
        assert!(shortcuts::is_supported_shortcut("#"));
        assert!(shortcuts::is_supported_shortcut("?"));

        assert!(shortcuts::is_supported_shortcut("!+"));
        assert!(shortcuts::is_supported_shortcut("+- extra"));
        assert!(shortcuts::is_supported_shortcut("  #?  "));

        assert!(!shortcuts::is_supported_shortcut("@"));
        assert!(!shortcuts::is_supported_shortcut("x"));
        assert!(!shortcuts::is_supported_shortcut(""));
        assert!(!shortcuts::is_supported_shortcut("   "));
    }

    #[test]
    fn test_get_shortcuts() {
        let shortcuts_list = shortcuts::get_shortcuts();

        assert!(shortcuts_list.contains("!+"));
        assert!(shortcuts_list.contains("!-"));
        assert!(shortcuts_list.contains("!#"));
        assert!(shortcuts_list.contains("++"));
        assert!(shortcuts_list.contains("+-"));
        assert!(shortcuts_list.contains("+#"));
        assert!(shortcuts_list.contains("-+"));
        assert!(shortcuts_list.contains("--"));
        assert!(shortcuts_list.contains("-#"));
        assert!(shortcuts_list.contains("#!"));
        assert!(shortcuts_list.contains("#+"));
        assert!(shortcuts_list.contains("#?"));
        assert!(shortcuts_list.contains("?!"));
        assert!(shortcuts_list.contains("?+"));
        assert!(shortcuts_list.contains("??"));
    }

    #[test]
    fn test_all_bang_shortcuts() {
        clear_log();

        assert!(shortcuts::dispatch("!+").is_ok());
        assert_eq!(get_calls("bang_plus").len(), 1);

        clear_log();
        assert!(shortcuts::dispatch("!-").is_ok());
        assert_eq!(get_calls("bang_minus").len(), 1);

        clear_log();
        assert!(shortcuts::dispatch("!#").is_ok());
        assert_eq!(get_calls("bang_hash").len(), 1);
    }

    #[test]
    fn test_all_plus_shortcuts() {
        clear_log();

        assert!(shortcuts::dispatch("++").is_ok());
        assert_eq!(get_calls("plus_plus").len(), 1);

        clear_log();
        assert!(shortcuts::dispatch("+-").is_ok());
        assert_eq!(get_calls("plus_minus").len(), 1);

        clear_log();
        assert!(shortcuts::dispatch("+#").is_ok());
        assert_eq!(get_calls("plus_hash").len(), 1);
    }

    #[test]
    fn test_all_minus_shortcuts() {
        clear_log();

        assert!(shortcuts::dispatch("-+").is_ok());
        assert_eq!(get_calls("minus_plus").len(), 1);

        clear_log();
        assert!(shortcuts::dispatch("--").is_ok());
        assert_eq!(get_calls("minus_minus").len(), 1);

        clear_log();
        assert!(shortcuts::dispatch("-#").is_ok());
        assert_eq!(get_calls("minus_hash").len(), 1);
    }

    #[test]
    fn test_all_hash_shortcuts() {
        clear_log();

        assert!(shortcuts::dispatch("#!").is_ok());
        assert_eq!(get_calls("hash_bang").len(), 1);

        clear_log();
        assert!(shortcuts::dispatch("#+").is_ok());
        assert_eq!(get_calls("hash_plus").len(), 1);

        clear_log();
        assert!(shortcuts::dispatch("#?").is_ok());
        assert_eq!(get_calls("hash_question").len(), 1);
    }

    #[test]
    fn test_all_question_shortcuts() {
        clear_log();

        assert!(shortcuts::dispatch("?!").is_ok());
        assert_eq!(get_calls("question_bang").len(), 1);

        clear_log();
        assert!(shortcuts::dispatch("?+").is_ok());
        assert_eq!(get_calls("question_plus").len(), 1);

        clear_log();
        assert!(shortcuts::dispatch("??").is_ok());
        assert_eq!(get_calls("question_question").len(), 1);
    }

    #[test]
    fn test_parameter_passing() {
        clear_log();

        shortcuts::dispatch("!+ first").unwrap();
        shortcuts::dispatch("!+ second").unwrap();
        shortcuts::dispatch("!+ third").unwrap();

        let calls = get_calls("bang_plus");
        assert_eq!(calls.len(), 3);
        assert_eq!(calls[0], "first");
        assert_eq!(calls[1], "second");
        assert_eq!(calls[2], "third");
    }

    #[test]
    fn test_hash_question_debug() {
        clear_log();

        // Test the exact failing case
        let input = "#? /path/to/file";
        let result = shortcuts::dispatch(input);

        // Check if dispatch succeeded
        assert!(result.is_ok(), "Dispatch failed with: {:?}", result);

        // Check if function was called
        let calls = get_calls("hash_question");
        assert!(!calls.is_empty(), "hash_question was not called at all");
        assert_eq!(
            calls.len(),
            1,
            "hash_question called {} times instead of 1",
            calls.len()
        );
        assert_eq!(
            calls[0], "/path/to/file",
            "Wrong parameter: got '{}' expected '/path/to/file'",
            calls[0]
        );
    }

    #[test]
    fn test_complex_parameters() {
        clear_log();

        shortcuts::dispatch("++ key=value").unwrap();
        assert_eq!(get_calls("plus_plus"), vec!["key=value"]);

        clear_log();
        shortcuts::dispatch("-- --flag").unwrap();
        assert_eq!(get_calls("minus_minus"), vec!["--flag"]);

        clear_log();
        shortcuts::dispatch("#+ /path/to/file").unwrap();
        assert_eq!(get_calls("hash_plus"), vec!["/path/to/file"]);

        clear_log();
        shortcuts::dispatch("?! 123 456 789").unwrap();
        assert_eq!(get_calls("question_bang"), vec!["123 456 789"]);
    }

    #[test]
    fn test_special_characters_in_parameters() {
        clear_log();

        shortcuts::dispatch("!+ @#$%").unwrap();
        assert_eq!(get_calls("bang_plus"), vec!["@#$%"]);

        clear_log();
        shortcuts::dispatch("?? !@#$%^&*()").unwrap();
        assert_eq!(get_calls("question_question"), vec!["!@#$%^&*()"]);

        clear_log();
        shortcuts::dispatch("+- hello!world?").unwrap();
        assert_eq!(get_calls("plus_minus"), vec!["hello!world?"]);
    }

    #[test]
    fn test_error_message_format() {
        let result = shortcuts::dispatch("xx");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Unknown shortcut"));
        assert!(err.contains("xx"));
    }

    #[test]
    fn test_sequential_dispatch() {
        clear_log();

        assert!(shortcuts::dispatch("!+ one").is_ok());
        assert!(shortcuts::dispatch("++ two").is_ok());
        assert!(shortcuts::dispatch("-- three").is_ok());
        assert!(shortcuts::dispatch("#? four").is_ok());

        assert_eq!(get_calls("bang_plus"), vec!["one"]);
        assert_eq!(get_calls("plus_plus"), vec!["two"]);
        assert_eq!(get_calls("minus_minus"), vec!["three"]);
        assert_eq!(get_calls("hash_question"), vec!["four"]);
    }

    #[test]
    fn test_unicode_parameters() {
        clear_log();

        shortcuts::dispatch("!+ 你好").unwrap();
        assert_eq!(get_calls("bang_plus"), vec!["你好"]);

        clear_log();
        shortcuts::dispatch("?? 🚀💻").unwrap();
        assert_eq!(get_calls("question_question"), vec!["🚀💻"]);
    }

    #[test]
    fn test_empty_vs_no_parameter() {
        clear_log();

        shortcuts::dispatch("!+").unwrap();
        assert_eq!(get_calls("bang_plus"), vec![""]);

        clear_log();
        shortcuts::dispatch("!+   ").unwrap();
        assert_eq!(get_calls("bang_plus"), vec![""]);
    }

    #[test]
    fn test_shortcut_boundary_cases() {
        // Test exactly 2 characters
        assert!(shortcuts::dispatch("!+").is_ok());

        // Test more than 2 characters (valid with param)
        assert!(shortcuts::dispatch("!+x").is_ok());

        // Test 1 character (invalid)
        assert!(shortcuts::dispatch("!").is_err());
    }
}
