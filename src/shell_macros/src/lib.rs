extern crate proc_macro;

mod commandsgen;
mod shortcutsgen;

use proc_macro::TokenStream;
use commandsgen::generate_commands_dispatcher_from_file;
use shortcutsgen::generate_shortcuts_dispatcher_from_file;

#[proc_macro]
pub fn generate_commands_dispatcher(input: TokenStream) -> TokenStream {
    generate_commands_dispatcher_from_file(input)
}

#[proc_macro]
pub fn generate_shortcuts_dispatcher(input: TokenStream) -> TokenStream {
    generate_shortcuts_dispatcher_from_file(input)
}

