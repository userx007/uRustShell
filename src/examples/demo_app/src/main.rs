use shell_config::{
    HISTORY_MAX_ENTRIES, HISTORY_TOTAL_CAPACITY, INPUT_MAX_LEN, MAX_HEXSTR_LEN, PROMPT,
};
use shell_engine::Shell;
use shell_macros::{generate_commands_dispatcher, generate_shortcuts_dispatcher};

use usercode::commands as uc;
use usercode::shortcuts as us;

generate_commands_dispatcher! {
    mod commands;
    hexstr_size = crate::MAX_HEXSTR_LEN;
    path = "../usercode/src/commands.cfg"
}

generate_shortcuts_dispatcher! {
    mod shortcuts;
    shortcut_size = crate::INPUT_MAX_LEN;
    path = "../usercode/src/shortcuts.cfg"
}

fn main() {
    Shell::<
        { commands::NUM_COMMANDS },
        { commands::MAX_FUNCTION_NAME_LEN },
        { INPUT_MAX_LEN },
        { HISTORY_TOTAL_CAPACITY },
        { HISTORY_MAX_ENTRIES },
        commands::DispatchError,
    >::new(
        commands::get_commands,
        commands::get_datatypes,
        shortcuts::get_shortcuts,
        shortcuts::is_supported_shortcut,
        commands::dispatch,
        shortcuts::dispatch,
        PROMPT,
    )
    .run();
}
