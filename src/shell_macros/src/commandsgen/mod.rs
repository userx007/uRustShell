#![allow(non_snake_case)]

//! # Command Dispatcher Macro
//!
//! This crate generates a no_std, zero-heap command dispatcher from a compact descriptor or mapping.
//!
//! ## Usage
//! - Accepts a module name and a descriptor string or mapping.
//! - Generates a module with a dispatcher, tokenizer, and helpers.
//!
//! ## Descriptor Table
//!
//! Each character in a descriptor represents one parameter type:

//! +------+-------+   +------+------+   +------+------+   +------+------+   +------+------+
//! | Char | Type  |   | Char | Type |   | Char | Type |   | Char | Type |   | Char | Type |
//! +------+-------+   +------+------+   +------+------+   +------+------+   +------+------+
//!
//! +------+-------+   +------+------+   +------+------+   +------+------+   +------+------+
//! | B    | u8    |   | W    | u16  |   | D    | u32  |   | Q    | u64  |   | X    | u128 |
//! +------+-------+   +------+------+   +------+------+   +------+------+   +------+------+
//! | b    | i8    |   | w    | i16  |   | d    | i32  |   | q    | i64  |   | x    | i128 |
//! +------+-------+   +------+------+   +------+------+   +------+------+   +------+------+
//!
//! +------+-------+   +------+------+   +------+------+   +------+------+   +------+------+
//! | Z    | usize |   | F    | f32  |   | c    | char |   | b    | bool |   | v    | void |
//! +------+-------+   +------+------+   +------+------+   +------+------+   +------+------+
//! | z    | isize |   | f    | f64  |   | s    | &str |   | h    | &[u8]|
 //!+------+-------+   +------+------+   +------+------+   +------+------+

//! Examples:
//! - "DdFsb" => arguments: u32, i32, f64, &str, bool
//! - "t"     => argument: bool
//! - "v"     => argument: void

//! ## Macro Input Format
//! - DSL: `define_commands!(mod m; \"dFs: path::to::f1 path::to::f2, t: path::to::flag\");`

//! * Tokenization splits a command line into tokens, respecting **double quotes** for `&str`.
//! * `dispatch(line)` parses the function name + arguments, checks **arity**, parses into a stack
//!   `CallCtx`, and invokes the registered function.
//! * No heap allocations are performed; buffers are compile-time sized from maximums inferred
//!   across all descriptors.
//! ## no_std
//! - Uses `core` only; suitable for embedded/stack-only use.

//! `DispatchError` reports: `Empty`, `UnknownFunction`, `WrongArity` and per-type parsing errors:
//! `BadBool`, `BadChar`, `BadUnsigned`, `BadSigned`, `BadFloat`.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{parse::Parse, parse_macro_input, Ident, LitStr, Result, Token};

/// A std-like alias used locally during macro input parsing.
type StdResult<T, E> = std::result::Result<T, E>;

/// Per-descriptor maximum counts of each primitive (used to size `CallCtx`).
#[derive(Default, Clone, Copy)]
struct HostCounts {

    // unsigned ints
    u8_c: usize, u16_c: usize, u32_c: usize, u64_c: usize, u128_c: usize,

    // signed ints
    i8_c: usize, i16_c: usize, i32_c: usize, i64_c: usize, i128_c: usize,

    // sized ints
    usize_c: usize, isize_c: usize,

    // floats
    f32_c: usize, f64_c: usize,

    // others
    bool_c: usize, char_c: usize, str_c: usize,

    // hexstring AABBF3C6 => [170, 187, 243, 198]
    hexstr_c: usize,
}

/// Component-wise maximum between two `HostCounts`.
fn host_counts_max(a: HostCounts, b: HostCounts) -> HostCounts {
    macro_rules! m { ($f:ident) => { if a.$f > b.$f { a.$f } else { b.$f } }; }
    HostCounts {
        u8_c: m!(u8_c),   u16_c: m!(u16_c),   u32_c: m!(u32_c),   u64_c: m!(u64_c),   u128_c: m!(u128_c),
        i8_c: m!(i8_c),   i16_c: m!(i16_c),   i32_c: m!(i32_c),   i64_c: m!(i64_c),   i128_c: m!(i128_c),
        usize_c: m!(usize_c), isize_c: m!(isize_c),
        f32_c: m!(f32_c), f64_c: m!(f64_c),
        bool_c: m!(bool_c), char_c: m!(char_c), str_c: m!(str_c), hexstr_c: m!(hexstr_c),
    }
}

/// Parsed macro input: `mod <ident>;` followed by either a DSL `LitStr`
struct CommandMacroInput {
    mod_ident: Ident,               // Module identifier for the generated dispatcher
    body: LitStr,                   // Macro input body as string
    hexstr_size: Option<syn::Expr>, // Optional size for hexstr buffers
}


/// Implementation for CommandMacroInput structure
impl Parse for CommandMacroInput {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        // Expect: `mod <ident>;`
        input.parse::<Token![mod]>()?;
        let mod_ident: Ident = input.parse()?;
        input.parse::<Token![;]>()?;

        // Optionally parse hexstr_size = <expr>;
        let hexstr_size = if input.peek(syn::Ident) && input.peek2(Token![=]) {
            let key: Ident = input.parse()?;
            if key == "hexstr_size" {
                input.parse::<Token![=]>()?;
                let expr: syn::Expr = input.parse()?;
                input.parse::<Token![;]>()?;
                Some(expr)
            } else {
                return Err(syn::Error::new(key.span(), "Unexpected identifier, expected 'hexstr_size'"));
            }
        } else {
            None
        };

        let body: LitStr = input.parse()?;
        Ok(CommandMacroInput { mod_ident, hexstr_size, body })
    }
}

/// Generate a no-heap dispatcher module from a DSL mapping.
pub fn define_commands_impl_(input: TokenStream) -> TokenStream {
    let CommandMacroInput { mod_ident, body, hexstr_size } = parse_macro_input!(input as CommandMacroInput);

    // Collect (descriptor, [paths]) pairs from either the DSL

    let mut pairs: Vec<(String, Vec<syn::Path>)> = {
            let s = body.value();
            let mut acc = Vec::new();
            for group in s.split(',') {
                let grp = group.trim();
                if grp.is_empty() { continue; }
                let (desc, names) = match grp.split_once(':') {
                    Some((d, r)) => (d.trim(), r.trim()),
                    None => continue,
                };
                if desc.is_empty() || names.is_empty() { continue; }
                let desc_str = desc.to_string();
                let funcs: StdResult<Vec<_>, _> = names
                    .split_whitespace()
                    .map(syn::parse_str::<syn::Path>)
                    .collect();
                let funcs = match funcs { Ok(v) => v, Err(_) => continue };
                acc.push((desc_str, funcs));
            }
            acc

    };

    // Deduplicate descriptors, assign indices, gather entries; stable sort by function name.
    let mut unique_desc: Vec<String> = Vec::new();
    let mut entries: Vec<FnEntry> = Vec::new();
    for (desc, funcs) in pairs.drain(..) {
        let idx = match unique_desc.iter().position(|x| x == &desc) {
            Some(i) => i,
            None => { unique_desc.push(desc.clone()); unique_desc.len() - 1 }
        };
        for p in funcs {
            let name_str = path_last_ident(&p).unwrap_or_else(|| "unknown".into());
            entries.push(FnEntry { name_str, path: p, spec: desc.clone(), spec_idx: idx });
        }
    }

    // Stable sort entries by function name
    entries.sort_by(|a, b| a.name_str.cmp(&b.name_str));

    // Get the largest name for a function
    let function_name_max_len = entries.iter().map(|e| e.name_str.len()).max().unwrap_or(0) + 1;

    // Human-readable registry of function names for diagnostics/UI.
    let fn_names: Vec<LitStr> = entries
        .iter()
        .map(|e| LitStr::new(&e.name_str, Span::call_site()))
        .collect();

    // Generated registry function
    let registry_fn = quote! {
        /// Return function names in the generated table (sorted).
        pub fn get_function_names() -> Vec<&'static str> {
            vec![ #( #fn_names ),* ]
        }
    };

    // Compute per-spec counts for each primitive type and the overall max arity.
    let mut max_counts = HostCounts::default();
    let mut max_arity: usize = 0;

    for desc in &unique_desc {
        let mut c = HostCounts::default();
        for ch in desc.chars() {
            match ch {

                // unsigned (lowercase)
                'B' => c.u8_c += 1,   // u8
                'W' => c.u16_c += 1,  // u16
                'D' => c.u32_c += 1,  // u32
                'Q' => c.u64_c += 1,  // u64
                'X' => c.u128_c += 1, // u128

                // signed (uppercase)
                'b' => c.i8_c += 1,   // i8
                'w' => c.i16_c += 1,  // i16
                'd' => c.i32_c += 1,  // i32
                'q' => c.i64_c += 1,  // i64
                'x' => c.i128_c += 1, // i128

                // sized
                'Z' => c.usize_c += 1, // usize
                'z' => c.isize_c += 1, // isize

                // floats
                'f' => c.f32_c += 1,  // f32
                'F' => c.f64_c += 1,  // f64

                // bool, char, string, hexstring
                't' => c.bool_c += 1, // bool
                'c' => c.char_c += 1, // char
                's' => c.str_c  += 1, // &str
                'h' => c.hexstr_c += 1, // hex &str

                // void
                'v' => {},
                _ => {}
            }
        }

        let arity = if desc == "v" {
            0
        } else {
            c.u8_c + c.u16_c + c.u32_c + c.u64_c + c.u128_c +
            c.i8_c + c.i16_c + c.i32_c + c.i64_c + c.i128_c +
            c.usize_c + c.isize_c +
            c.f32_c + c.f64_c +
            c.bool_c + c.char_c + c.str_c + c.hexstr_c
        };

        if arity > max_arity { max_arity = arity; }
        max_counts = host_counts_max(max_counts, c);
    }

    // Keep raw descriptor strings for diagnostics in the generated module.
    let param_specs: Vec<LitStr> = unique_desc
        .iter()
        .map(|s| LitStr::new(s, Span::call_site()))
        .collect();
    let param_specs_len = param_specs.len();

    // Generate maximals as constants
    let max_u8      = max_counts.u8_c;
    let max_u16     = max_counts.u16_c;
    let max_u32     = max_counts.u32_c;
    let max_u64     = max_counts.u64_c;
    let max_u128    = max_counts.u128_c;
    let max_i8      = max_counts.i8_c;
    let max_i16     = max_counts.i16_c;
    let max_i32     = max_counts.i32_c;
    let max_i64     = max_counts.i64_c;
    let max_i128    = max_counts.i128_c;
    let max_usize   = max_counts.usize_c;
    let max_isize   = max_counts.isize_c;
    let max_f32     = max_counts.f32_c;
    let max_f64     = max_counts.f64_c;
    let max_bool    = max_counts.bool_c;
    let max_char    = max_counts.char_c;
    let max_str     = max_counts.str_c;
    let max_hexstr  = max_counts.hexstr_c;
    let max_arity_num = max_arity;

    // Generate per-descriptor parsers that fill `CallCtx` from `&[&str]`.
    let mut parsers: Vec<TokenStream2> = Vec::new();
    for (sid, spec) in unique_desc.iter().enumerate() {
        let fn_ident = format_ident!("__parse_spec_{}", sid);
        let header = quote! {
            // `k` indexes into the argument tokens slice; individual idx_* track per-type positions.
            let mut k = 0usize;
            // per-type indices
            let mut idx_b=0usize; let mut idx_w=0usize; let mut idx_d=0usize; let mut idx_q=0usize; let mut idx_x=0usize;
            let mut idx_B=0usize; let mut idx_W=0usize; let mut idx_D=0usize; let mut idx_Q=0usize; let mut idx_X=0usize;
            let mut idx_z=0usize; let mut idx_Z=0usize;
            let mut idx_f=0usize; let mut idx_F=0usize;
            let mut idx_t=0usize; let mut idx_c=0usize; let mut idx_s=0usize; let mut idx_h=0usize;
        };

        let mut stmts: Vec<TokenStream2> = Vec::new();
        for ch in spec.chars() {
            let stmt = match ch {
                // unsigned
                'B' => quote! { ctx.u8s   [idx_b] = parse_u8   (args[k]).ok_or(DispatchError::BadUnsigned)?; idx_b+=1; k+=1; },
                'W' => quote! { ctx.u16s  [idx_w] = parse_u16  (args[k]).ok_or(DispatchError::BadUnsigned)?; idx_w+=1; k+=1; },
                'D' => quote! { ctx.u32s  [idx_d] = parse_u32  (args[k]).ok_or(DispatchError::BadUnsigned)?; idx_d+=1; k+=1; },
                'Q' => quote! { ctx.u64s  [idx_q] = parse_u64  (args[k]).ok_or(DispatchError::BadUnsigned)?; idx_q+=1; k+=1; },
                'X' => quote! { ctx.u128s [idx_x] = parse_u128 (args[k]).ok_or(DispatchError::BadUnsigned)?; idx_x+=1; k+=1; },
                // signed
                'b' => quote! { ctx.i8s   [idx_B] = parse_i8   (args[k]).ok_or(DispatchError::BadSigned  )?; idx_B+=1; k+=1; },
                'w' => quote! { ctx.i16s  [idx_W] = parse_i16  (args[k]).ok_or(DispatchError::BadSigned  )?; idx_W+=1; k+=1; },
                'd' => quote! { ctx.i32s  [idx_D] = parse_i32  (args[k]).ok_or(DispatchError::BadSigned  )?; idx_D+=1; k+=1; },
                'q' => quote! { ctx.i64s  [idx_Q] = parse_i64  (args[k]).ok_or(DispatchError::BadSigned  )?; idx_Q+=1; k+=1; },
                'x' => quote! { ctx.i128s [idx_X] = parse_i128 (args[k]).ok_or(DispatchError::BadSigned  )?; idx_X+=1; k+=1; },
                // sized
                'Z' => quote! { ctx.usizes[idx_z] = parse_usize(args[k]).ok_or(DispatchError::BadUnsigned)?; idx_z+=1; k+=1; },
                'z' => quote! { ctx.isizes[idx_Z] = parse_isize(args[k]).ok_or(DispatchError::BadSigned  )?; idx_Z+=1; k+=1; },
                // floats
                'f' => quote! { ctx.f32s  [idx_f] = parse_f::<f32  >(args[k]).ok_or(DispatchError::BadFloat)?; idx_f+=1; k+=1; },
                'F' => quote! { ctx.f64s  [idx_F] = parse_f::<f64  >(args[k]).ok_or(DispatchError::BadFloat)?; idx_F+=1; k+=1; },
                //  bool, char, string, hexstring
                't' => quote! { ctx.bools [idx_t] = parse_bool(args[k]).ok_or(DispatchError::BadBool)?; idx_t+=1; k+=1; },
                'c' => quote! { ctx.chars [idx_c] = parse_char(args[k]).ok_or(DispatchError::BadChar)?; idx_c+=1; k+=1; },
                's' => quote! { ctx.strs  [idx_s] = args[k]; idx_s+=1; k+=1; },
                'h' => quote! { ctx.hexstrs[idx_h]= parse_hexstr(args[k]).ok_or(DispatchError::BadHexStr)?; idx_h+=1; k+=1; },
                _   => quote! {},
            };
            stmts.push(stmt);
        }
        parsers.push(quote! {

            /// Parse arguments for this descriptor into `CallCtx`.
            #[inline(always)]
            fn #fn_ident<'a>(ctx: &mut CallCtx<'a>, args: &[&'a str]) -> Result<(), DispatchError> {
                #header
                #(#stmts)*
                Ok(())
            }
        });
    }

    // Generate per-function wrappers and entries + match arms for lookup
    let mut wrappers: Vec<TokenStream2> = Vec::new();
    let mut entry_inits: Vec<TokenStream2> = Vec::new();
    let mut match_arms: Vec<TokenStream2> = Vec::new();

    // Pairs of (function name, descriptor) for diagnostics / UI
    let name_spec_pairs: Vec<TokenStream2> = entries.iter().map(|e| {
        let name_lit = LitStr::new(&e.name_str, Span::call_site());
        let spec_lit = LitStr::new(&e.spec,      Span::call_site());
        quote! { (#name_lit, #spec_lit) }
    }).collect();

    for (pos, e) in entries.iter().enumerate() {
        let name_lit = LitStr::new(&e.name_str, Span::call_site());
        let spec_str = &e.spec;
        //let arity_u8 = (spec_str.chars().count()) as u8;
        let arity_u8 = if spec_str == "v" { 0 } else { spec_str.chars().count() as u8 };
        let wrapper_ident = format_ident!("__call_{}", sanitize_ident(&e.name_str));
        let path = &e.path;
        let spec_idx_u16 = e.spec_idx as u16;
        let parser_ident = format_ident!("__parse_spec_{}", e.spec_idx);

        // Build type list and extraction expressions according to the descriptor order.
        let mut arg_types: Vec<TokenStream2> = Vec::new();
        let mut arg_exprs: Vec<TokenStream2> = Vec::new();
        let mut idx_b=0usize; let mut idx_w=0usize; let mut idx_d=0usize; let mut idx_q=0usize; let mut idx_x=0usize;
        let mut idx_B=0usize; let mut idx_W=0usize; let mut idx_D=0usize; let mut idx_Q=0usize; let mut idx_X=0usize;
        let mut idx_z=0usize; let mut idx_Z=0usize;
        let mut idx_f=0usize; let mut idx_F=0usize;
        let mut idx_t=0usize; let mut idx_c=0usize; let mut idx_s=0usize; let mut idx_h=0usize;

        for ch in spec_str.chars() {
            match ch {

                // unsigned
                'B' => { arg_types.push(quote!{ u8    }); arg_exprs.push(quote!{ ctx.u8s    [#idx_b] }); idx_b+=1; }
                'W' => { arg_types.push(quote!{ u16   }); arg_exprs.push(quote!{ ctx.u16s   [#idx_w] }); idx_w+=1; }
                'D' => { arg_types.push(quote!{ u32   }); arg_exprs.push(quote!{ ctx.u32s   [#idx_d] }); idx_d+=1; }
                'Q' => { arg_types.push(quote!{ u64   }); arg_exprs.push(quote!{ ctx.u64s   [#idx_q] }); idx_q+=1; }
                'X' => { arg_types.push(quote!{ u128  }); arg_exprs.push(quote!{ ctx.u128s  [#idx_x] }); idx_x+=1; }

                // signed
                'b' => { arg_types.push(quote!{ i8    }); arg_exprs.push(quote!{ ctx.i8s    [#idx_B] }); idx_B+=1; }
                'w' => { arg_types.push(quote!{ i16   }); arg_exprs.push(quote!{ ctx.i16s   [#idx_W] }); idx_W+=1; }
                'd' => { arg_types.push(quote!{ i32   }); arg_exprs.push(quote!{ ctx.i32s   [#idx_D] }); idx_D+=1; }
                'q' => { arg_types.push(quote!{ i64   }); arg_exprs.push(quote!{ ctx.i64s   [#idx_Q] }); idx_Q+=1; }
                'x' => { arg_types.push(quote!{ i128  }); arg_exprs.push(quote!{ ctx.i128s  [#idx_X] }); idx_X+=1; }

                // sized
                'Z' => { arg_types.push(quote!{ usize }); arg_exprs.push(quote!{ ctx.usizes [#idx_z] }); idx_z+=1; }
                'z' => { arg_types.push(quote!{ isize }); arg_exprs.push(quote!{ ctx.isizes [#idx_Z] }); idx_Z+=1; }

                // floats
                'f' => { arg_types.push(quote!{ f32   }); arg_exprs.push(quote!{ ctx.f32s   [#idx_f] }); idx_f+=1; }
                'F' => { arg_types.push(quote!{ f64   }); arg_exprs.push(quote!{ ctx.f64s   [#idx_F] }); idx_F+=1; }

                // others
                't' => { arg_types.push(quote!{ bool  }); arg_exprs.push(quote!{ ctx.bools  [#idx_t] }); idx_t+=1; }
                'c' => { arg_types.push(quote!{ char  }); arg_exprs.push(quote!{ ctx.chars  [#idx_c] }); idx_c+=1; }
                's' => { arg_types.push(quote!{ &str  }); arg_exprs.push(quote!{ ctx.strs   [#idx_s] }); idx_s+=1; }
                'h' => { arg_types.push(quote!{ &[u8] }); arg_exprs.push(quote!{ &ctx.hexstrs[#idx_h] }); idx_h+=1; }
                _ => {}
            }
        }

        // Compile-time signature check: ensures `path` has the expected arity/types.
        let sig_check = {
            let fn_type = quote! { fn(#(#arg_types),*) -> _ };
            quote! {
                const _: fn() = || {
                    let _check: #fn_type = #path;
                    let _ = _check;
                };
            }
        };

        wrappers.push(quote! {
            #sig_check

            /// Wrapper that extracts arguments from `CallCtx` and calls the target function.
            #[inline(always)]
            fn #wrapper_ident<'__ctx>(ctx: &mut CallCtx<'__ctx>, _av: ArgsView<'__ctx>) -> Result<(), DispatchError> {
                let _ = #path( #(#arg_exprs),* );
                Ok(())
            }
        });

        entry_inits.push(quote! {
            Entry {
                name: #name_lit,
                arity: #arity_u8,
                parser: #parser_ident,
                caller: #wrapper_ident,
                spec_idx: #spec_idx_u16,
            }
        });

        match_arms.push(quote! { #name_lit => Some(&ENTRIES[#pos]), });
    }

    let max_hexstr_len_expr = if let Some(expr) = &hexstr_size {
        quote! { #expr }
    } else {
        // Emit a compile error at macro expansion time
        return syn::Error::new(
            Span::call_site(),
            "You must provide `hexstr_size = ...;` in the macro input."
        ).to_compile_error().into();
    };

    let out = quote! {
        #[allow(dead_code)]
        #[allow(non_snake_case, non_camel_case_types, unused_imports)]
        pub mod #mod_ident {

            //! Generated by `define_commands!`. See the macro docs for usage and the descriptor table.
            extern crate core;

            // Macro and parse functions for integer parsing with base detection
            macro_rules! parse_int {
                ($name:ident, $ty:ty) => {
                    fn $name(s: &str) -> Option<$ty> {
                        let s = s.trim();
                        if let Some(stripped) = s.strip_prefix("0x") {
                            <$ty>::from_str_radix(stripped, 16).ok()
                        } else if let Some(stripped) = s.strip_prefix("0o") {
                            <$ty>::from_str_radix(stripped, 8).ok()
                        } else if let Some(stripped) = s.strip_prefix("0b") {
                            <$ty>::from_str_radix(stripped, 2).ok()
                        } else {
                            s.parse::<$ty>().ok()
                        }
                    }
                };
            }

            parse_int!(parse_u8, u8);
            parse_int!(parse_u16, u16);
            parse_int!(parse_u32, u32);
            parse_int!(parse_u64, u64);
            parse_int!(parse_u128, u128);

            parse_int!(parse_i8, i8);
            parse_int!(parse_i16, i16);
            parse_int!(parse_i32, i32);
            parse_int!(parse_i64, i64);
            parse_int!(parse_i128, i128);

            parse_int!(parse_usize, usize);
            parse_int!(parse_isize, isize);

            /// All unique parameter descriptors encountered (for diagnostics/UIs).
            pub static PARAM_SPECS: [&'static str; #param_specs_len] = [ #( #param_specs ),* ];

            /// Descriptor character to Rust type mapping (for help/diagnostics).
            pub static DESCRIPTOR_HELP: &str = "B:u8   | W:u16  | D:u32 | Q:u64 | X:u128 | Z:usize | F:f64\nb:i8   | w:i16  | d:i32 | q:i64 | x:i128 | z:isize | f:f32\nv:void | c:char | s:str | t:bool | h:hexstr\n";

            /// Maximum counts per primitive across all descriptors. These sizes define the
            pub const MAX_U8:    usize = #max_u8;
            pub const MAX_U16:   usize = #max_u16;
            pub const MAX_U32:   usize = #max_u32;
            pub const MAX_U64:   usize = #max_u64;
            pub const MAX_U128:  usize = #max_u128;

            pub const MAX_I8:    usize = #max_i8;
            pub const MAX_I16:   usize = #max_i16;
            pub const MAX_I32:   usize = #max_i32;
            pub const MAX_I64:   usize = #max_i64;
            pub const MAX_I128:  usize = #max_i128;

            pub const MAX_USIZE: usize = #max_usize;
            pub const MAX_ISIZE: usize = #max_isize;

            pub const MAX_F32:   usize = #max_f32;
            pub const MAX_F64:   usize = #max_f64;

            pub const MAX_BOOL:  usize = #max_bool;
            pub const MAX_CHAR:  usize = #max_char;
            pub const MAX_HEXSTR:usize = #max_hexstr;
            pub const MAX_STR:   usize = #max_str;
            pub const MAX_HEXSTR_LEN: usize = #max_hexstr_len_expr;

            /// Maximum arity across all functions; token buffers use `1 + MAX_ARITY`.
            pub const MAX_ARITY: usize = #max_arity_num;

            /// Maximum number of commands
            pub const NUM_COMMANDS: usize = ENTRIES.len();

            // Largest function name
            pub const MAX_FUNCTION_NAME_LEN: usize = #function_name_max_len;

            /// One entry per function available to the dispatcher.
            pub struct Entry {

                /// Function name used in textual calls (first token).
                pub name: &'static str,

                /// Required positional arity.
                pub arity: u8,

                /// Descriptor-specific parser filling `CallCtx` from `&[&str]`.
                pub parser: for<'ctx> fn(&mut CallCtx<'ctx>, &[&'ctx str]) -> Result<(), DispatchError>,

                /// Wrapper invoking the target function.
                pub caller: for<'ctx> fn(&mut CallCtx<'ctx>, ArgsView<'ctx>) -> Result<(), DispatchError>,

                /// Index into `PARAM_SPECS` (for diagnostics).
                pub spec_idx: u16,
            }

            /// A lightweight view over the raw tokens for advanced callers.
            pub struct ArgsView<'a> {
                pub tokens: &'a [&'a str],
                pub len: usize,
            }

            /// Errors Generateted by tokenization, arity check, or per-type parsing.
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum DispatchError {

                /// Input line contains no tokens.
                Empty,

                /// No function with the given name exists in the table.
                UnknownFunction,

                /// Function exists, but arity mismatched.
                WrongArity { expected: u8 },

                /// Failed to parse a `bool`.
                BadBool,

                /// Failed to parse a `char` (must be exactly one Unicode scalar).
                BadChar,

                /// Failed to parse an unsigned integer (`u*`).
                BadUnsigned,

                /// Failed to parse a signed integer (`i*`).
                BadSigned,

                /// Failed to parse a float (`f64`).
                BadFloat,

                /// Failed to parse a hexlified string.
                BadHexStr,
            }

            /// Stack-only argument storage sized by the `MAX_*` constants.
            pub struct CallCtx<'a> {
                pub u8s:    [u8;    MAX_U8],
                pub u16s:   [u16;   MAX_U16],
                pub u32s:   [u32;   MAX_U32],
                pub u64s:   [u64;   MAX_U64],
                pub u128s:  [u128;  MAX_U128],

                pub i8s:    [i8;    MAX_I8],
                pub i16s:   [i16;   MAX_I16],
                pub i32s:   [i32;   MAX_I32],
                pub i64s:   [i64;   MAX_I64],
                pub i128s:  [i128;  MAX_I128],

                pub usizes: [usize; MAX_USIZE],
                pub isizes: [isize; MAX_ISIZE],

                pub f32s:   [f32;   MAX_F32],
                pub f64s:   [f64;   MAX_F64],

                pub bools:  [bool;  MAX_BOOL],
                pub chars:  [char;  MAX_CHAR],
                pub strs:   [&'a str; MAX_STR],
                pub hexstrs: [heapless::Vec<u8, MAX_HEXSTR_LEN>; MAX_HEXSTR],
            }

            impl<'a> CallCtx<'a> {
                /// Construct a zero-initialized `CallCtx`.
                #[inline(always)]
                pub fn new() -> Self {
                    Self {
                        u8s:    [0;    MAX_U8],
                        u16s:   [0;    MAX_U16],
                        u32s:   [0;    MAX_U32],
                        u64s:   [0;    MAX_U64],
                        u128s:  [0;    MAX_U128],

                        i8s:    [0;    MAX_I8],
                        i16s:   [0;    MAX_I16],
                        i32s:   [0;    MAX_I32],
                        i64s:   [0;    MAX_I64],
                        i128s:  [0;    MAX_I128],

                        usizes: [0;    MAX_USIZE],
                        isizes: [0;    MAX_ISIZE],

                        f32s:   [0.0;  MAX_F32],
                        f64s:   [0.0;  MAX_F64],

                        bools:  [false; MAX_BOOL],
                        chars:  ['\0'; MAX_CHAR],
                        strs:   ["";   MAX_STR],
                        hexstrs: core::array::from_fn(|_| heapless::Vec::new()),
                    }
                }
            }

            /// Generated per-spec parsers
            #( #parsers )*

            /// Generated per-function wrappers
            #( #wrappers )*

            /// Function registry and lookup
            #registry_fn

            /// Static function table (sorted by name).
            pub static ENTRIES: &[Entry] = &[
                #( #entry_inits ),*
            ];

            /// Fast string-table lookup (match on string literal).
            #[inline(always)]
            fn find_entry(name: &str) -> Option<&'static Entry> {
                match name {
                    #( #match_arms )*
                    _ => None,
                }
            }

            /// Static pairs of (function name, parameter descriptor).
            pub static NAME_AND_SPEC: &[(&'static str, &'static str)] = &[
                #( #name_spec_pairs ),*
            ];

            /// Return (function name, descriptor) pairs. No allocations.
            #[inline(always)]
            pub fn get_commands() -> &'static [(&'static str, &'static str)] {
                NAME_AND_SPEC
            }

            /// Return descriptor help string (character to type mapping).
            #[inline(always)]
            pub fn get_datatypes() -> &'static str {
                DESCRIPTOR_HELP
            }

            /// Parse a hexlified string (even-length, non-empty, valid hex).
            #[inline(always)]
            pub fn parse_hexstr(s: &str) -> Option<heapless::Vec<u8, MAX_HEXSTR_LEN>> {
                if s.len() % 2 != 0 || s.is_empty() || (s.len() / 2) > MAX_HEXSTR_LEN {
                    return None;
                }
                (0..s.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&s[i..i+2], 16).ok())
                    .collect()
            }

            // Quotes-aware tokenizer (no heap). Caller provides the buffer.
            /// Splits by ASCII space or tab. A pair of `"` quotes groups a token (quotes
            /// Returns `Empty` if no tokens were produced.
            pub fn tokenize<'a>(line: &'a str, out: &mut [&'a str]) -> Result<usize, DispatchError> {
                let bytes = line.as_bytes();
                let mut i = 0usize;
                let mut n = 0usize;

                while i < bytes.len() {
                    // Skip leading spaces
                    while i < bytes.len() && is_space(bytes[i]) { i += 1; }
                    if i >= bytes.len() { break; }

                    if bytes[i] == b'"' {
                        // Quoted token
                        let start = i + 1;
                        i = start;
                        while i < bytes.len() && bytes[i] != b'"' { i += 1; }
                        if n < out.len() { out[n] = &line[start..i]; n += 1; }
                        if i < bytes.len() { i += 1; }
                        // Consume trailing non-space until next whitespace to match original behavior.
                        while i < bytes.len() && !is_space(bytes[i]) { i += 1; }
                    } else {
                        // Unquoted token
                        let start = i;
                        while i < bytes.len() && !is_space(bytes[i]) { i += 1; }
                        if n < out.len() { out[n] = &line[start..i]; n += 1; }
                    }
                }

                if n == 0 { return Err(DispatchError::Empty); }
                Ok(n)
            }

            /// ASCII space or tab.
            #[inline(always)]
            const fn is_space(b: u8) -> bool { b == b' ' || b == b'\t' }

            /// Accepts `1|true|True|TRUE` as `true`, and `0|false|False|FALSE` as `false`.
            #[inline(always)]
            fn parse_bool(s: &str) -> Option<bool> {
                match s {
                    "1" | "true" | "True" | "TRUE" => Some(true),
                    "0" | "false" | "False" | "FALSE" => Some(false),
                    _ => None,
                }
            }

            /// One-character string => `char`.
            #[inline(always)]
            fn parse_char(s: &str) -> Option<char> {
                let mut it = s.chars();
                let c = it.next()?;
                if it.next().is_none() { Some(c) } else { None }
            }

            #[inline(always)]
            fn parse_f<T>(s: &str) -> Option<T> where T: core::str::FromStr { s.parse::<T>().ok() }

            #[inline(always)]
            pub fn dispatch(line: &str) -> Result<(), DispatchError> {
                // + 2 in order to detect if more args than expected are provided..
                let mut toks: [&str; 2 + MAX_ARITY] = [""; 2 + MAX_ARITY];
                dispatch_with_buf(line, &mut toks)
            }

            /// Embedded-friendly entry point: caller supplies the token buffer.
            #[inline(always)]
            pub fn dispatch_with_buf<'a>(line: &'a str, toks: &mut [&'a str]) -> Result<(), DispatchError> {
                let len = tokenize(line, toks)?;
                let name = toks[0];
                let got_arity = (len - 1) as u16;
                let ent = find_entry(name).ok_or(DispatchError::UnknownFunction)?;
                if got_arity != ent.arity as u16 {
                    return Err(DispatchError::WrongArity { expected: ent.arity });
                }

                // Fill CallCtx from raw &str tokens (no heap).
                let mut ctx = CallCtx::new();
                let args_tokens: &[&str] = &toks[1..len];
                (ent.parser)(&mut ctx, args_tokens)?;

                // Provide a view for advanced use (currently unused by wrappers).
                let args = ArgsView { tokens: args_tokens, len: len - 1 };
                (ent.caller)(&mut ctx, args)
            }
        }
    };

    out.into()
}

/// Internal representation of one function to register (pre-codegen).
struct FnEntry {
    name_str: String,
    path: syn::Path,
    spec: String,
    spec_idx: usize,
}

/// Last path segment (function ident) as a `String`.
fn path_last_ident(p: &syn::Path) -> Option<String> {
    p.segments.last().map(|s| s.ident.to_string())
}

/// Make a valid identifier for wrapper functions (replace non-ASCII-alnum with `_`).
fn sanitize_ident(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect()
}


pub fn define_commands_impl(input: TokenStream) -> TokenStream {
    use syn::{parse::ParseStream, Expr};

    struct FileMacroInput {
        _mod_token: Token![mod],      // Token for `mod` keyword
        mod_name: Ident,              // Name of the module to generate
        _semi1: Token![;],            // Semicolon after module declaration
        _hexstr_size_token: Ident,    // Identifier for hexstr_size
        _eq_token: Token![=],         // Equals token for hexstr_size assignment
        hexstr_size: Expr,            // Expression for hexstr_size value
        _semi2: Token![;],            // Semicolon after hexstr_size assignment
        _path_token: Ident,           // Identifier for path
        _eq_token2: Token![=],        // Equals token for path assignment
        path: LitStr,                 // Literal string for file path
    }

    impl Parse for FileMacroInput {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            Ok(FileMacroInput {
                _mod_token: input.parse()?,
                mod_name: input.parse()?,
                _semi1: input.parse()?,
                _hexstr_size_token: input.parse()?,
                _eq_token: input.parse()?,
                hexstr_size: input.parse()?,
                _semi2: input.parse()?,
                _path_token: input.parse()?,
                _eq_token2: input.parse()?,
                path: input.parse()?,
            })
        }
    }

    let FileMacroInput {
        mod_name,
        hexstr_size,
        path,
        ..
    } = parse_macro_input!(input as FileMacroInput);

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let full_path = std::path::Path::new(&manifest_dir).join(path.value());


    let raw_dsl = std::fs::read_to_string(&full_path)
        .expect(&format!("Failed to read command descriptor file: {:?}", full_path));

    let macro_input = quote! {
        mod #mod_name;
        hexstr_size = #hexstr_size;
        #raw_dsl
    };

    define_commands_impl_(macro_input.into())
}
