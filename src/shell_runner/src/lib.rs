use heapless::String;

use shell_config::{PROMPT, INPUT_MAX_LEN, HISTORY_TOTAL_CAPACITY, HISTORY_MAX_ENTRIES, MAX_HEXSTR_LEN};
use shell_core::input::parser::InputParser;
use shell_core::terminal::RawMode;
use shell_macros::{define_shortcuts, define_commands};

use usercode::commands as uc;
use usercode::shortcuts as us;


define_commands!{
    mod commands;
    hexstr_size = crate::MAX_HEXSTR_LEN;
    path = "../usercode/src/commands.cfg"
}

define_shortcuts!{
    mod shortcuts;
    shortcut_size = crate::INPUT_MAX_LEN;
    path = "../usercode/src/shortcuts.cfg"
}


pub struct Shell<'a> {
    parser: InputParser<'a,{commands::NUM_COMMANDS},{commands::MAX_FUNCTION_NAME_LEN},INPUT_MAX_LEN,HISTORY_TOTAL_CAPACITY,HISTORY_MAX_ENTRIES>,
    _terminal : RawMode,
}

impl Shell<'_> {
    pub fn new() -> Self {
        let shell_commands  = commands::get_commands();
        let shell_datatypes = commands::get_datatypes();
        let shell_shortcuts = shortcuts::get_shortcuts();
        let parser          = InputParser::<{commands::NUM_COMMANDS},{commands::MAX_FUNCTION_NAME_LEN},INPUT_MAX_LEN,HISTORY_TOTAL_CAPACITY,HISTORY_MAX_ENTRIES>::new(shell_commands, shell_datatypes, shell_shortcuts, PROMPT);

        Self {
            parser,
            _terminal : RawMode::new(0),
        }
    }

    pub fn run(&mut self) {
        loop {
            if !self.parser.parse_input(Self::exec) {
                println!("⛔ Shell exited...");
                break;
            }
        }
    }

    fn exec(input: &String<INPUT_MAX_LEN>) {
        let result: Result<(), String<INPUT_MAX_LEN>> = if shortcuts::is_supported_shortcut(input) {
            shortcuts::dispatch(input)
        } else {
            commands::dispatch(input).map_err(|e| {
                let mut err_str = String::<INPUT_MAX_LEN>::new();
                use core::fmt::Write;
                write!(&mut err_str, "{:?}", e).unwrap();
                err_str
            })
        };

        match result {
            Ok(_) => println!("✅ Success: {}", input),
            Err(e) => println!("❌ Error: {} for line '{}'", e, input),
        }
    }

}

impl Default for Shell<'_> {
    fn default() -> Self {
        Self::new()
    }
}