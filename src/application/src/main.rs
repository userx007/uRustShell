mod cmd_impl;
mod shortcut_impl;

use shortcut_dispatcher::define_shortcuts;
use cmd_dispatcher::define_commands;
use input_parser::InputParser;

// Updated descriptors to the new mapping (case = signedness, width letters)
define_commands! {
    mod commands;
    r#"
    bdDtq : crate::cmd_impl::testfct,
    DDDDD : crate::cmd_impl::testi,
    s     : crate::cmd_impl::greet
            crate::cmd_impl::greet_again,
    wFs   : crate::cmd_impl::parse_mix,
    v     : crate::cmd_impl::vtest,
    ss    : crate::cmd_impl::greet2
    "#
}

define_shortcuts!{
    mod shortcuts;
    r#"
    + : { + : crate::shortcut_impl::shortcut_plus_plus,
          l : crate::shortcut_impl::shortcut_plus_l,
          m : crate::shortcut_impl::shortcut_plus_m,
          ? : crate::shortcut_impl::shortcut_plus_question_mark,
          ~ : crate::shortcut_impl::shortcut_plus_tilde
        },

    . : { . : crate::shortcut_impl::shortcut_dot_dot,
          z : crate::shortcut_impl::shortcut_dot_z,
          k : crate::shortcut_impl::shortcut_dot_k
        },

    - : { . : crate::shortcut_impl::shortcut_minus_dot,
          t : crate::shortcut_impl::shortcut_minus_t,
          u : crate::shortcut_impl::shortcut_minus_u,
          w : crate::shortcut_impl::shortcut_minus_w
        },
    "#
}


fn main() {

    let cmd_specs = commands::get_cmd_specs();
    let types_info = commands::get_descriptor_help();
    let shortcuts_info = shortcuts::list_supported_shortcuts();

    let mut parser = InputParser::new(cmd_specs, types_info, shortcuts_info);
    println!("Items:{}", cmd_specs.len());

    print!("\n❗Type '#q' to exit❗\n");

    loop {
        if let Some(input) = parser.parse_input() {
            if !input.is_empty() {
                if shortcuts::is_supported_shortcut(&input) {
                    match shortcuts::dispatch(&input) {
                        Ok(_) => print!("✅ Success: {}", input),
                        Err(e) => print!("❌ Error: {:?} for line '{}'", e, input),
                    }
                    continue;
                }
                match commands::dispatch(&input) {
                    Ok(_) => print!("✅ Success: {}", input),
                    Err(e) => print!("❌ Error: {:?} for line '{}'", e, input),
                }
            }
        } else {
            println!("❗Exiting...");
            break;
        }
    }
}