use ushell_config::*;
use ushell_dispatcher::{generate_commands_dispatcher, generate_shortcuts_dispatcher};
use ushell2::uShell;

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
    uShell::<
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
