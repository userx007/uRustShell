extern crate proc_macro;

mod commandsgen;
mod shortcutsgen;

use proc_macro::TokenStream;
use commandsgen::define_commands_impl;
use shortcutsgen::define_shortcuts_impl;

#[proc_macro]
pub fn define_commands(input: TokenStream) -> TokenStream {
    define_commands_impl(input)
}

#[proc_macro]
pub fn define_shortcuts(input: TokenStream) -> TokenStream {
    define_shortcuts_impl(input)
}

