

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitStr, Token,
};

// Struct to parse macro input: `mod <name>; <string>`
#[allow(dead_code)]
struct ShortcutMacroInput {
    mod_token: Token![mod],
    mod_name: Ident,
    semi: Token![;],
    content: LitStr,
}

impl Parse for ShortcutMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ShortcutMacroInput {
            mod_token: input.parse()?,
            mod_name: input.parse()?,
            semi: input.parse()?,
            content: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn define_shortcuts(input: TokenStream) -> TokenStream {
    let ShortcutMacroInput { mod_name, content, .. } =
        parse_macro_input!(input as ShortcutMacroInput);
    let raw = content.value();
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
        pub fn list_supported_shortcuts() -> &'static str {
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
        pub fn dispatch(input: &str) -> Result<(), String> {
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
                _ => Err(format!("Unknown shortcut: {}", key)),
            }
        }
    };

    let expanded = quote! {
        pub mod #mod_name {
            #dispatch_fn
            #support_fn
            #list_fn
        }
    };

    TokenStream::from(expanded)
}