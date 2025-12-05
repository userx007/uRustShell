use core::fmt::Debug;
use heapless::String;

use ushell_input::input::parser::InputParser;
use ushell_input::terminal::RawMode;

#[allow(non_camel_case_types)]
pub struct uShell<
    const NC: usize,
    const FNL: usize,
    const IML: usize,
    const HTC: usize,
    const HME: usize,
    ERRTYPE: Debug,
> {
    parser: InputParser<'static, NC, FNL, IML, HTC, HME>,
    _terminal: RawMode,
    is_shortcut: fn(&str) -> bool,
    command_dispatcher: fn(&str) -> Result<(), ERRTYPE>,
    shortcut_dispatcher: fn(&str) -> Result<(), heapless::String<IML>>,
}

impl<
    const NC: usize,
    const FNL: usize,
    const IML: usize,
    const HTC: usize,
    const HME: usize,
    ERRTYPE: Debug,
> uShell<NC, FNL, IML, HTC, HME, ERRTYPE>
{
    pub fn new(
        get_commands: fn() -> &'static [(&'static str, &'static str)],
        get_datatypes: fn() -> &'static str,
        get_shortcuts: fn() -> &'static str,
        is_shortcut: fn(&str) -> bool,
        command_dispatcher: fn(&str) -> Result<(), ERRTYPE>,
        shortcut_dispatcher: fn(&str) -> Result<(), heapless::String<IML>>,
        prompt: &'static str,
    ) -> Self {
        let parser = InputParser::<NC, FNL, IML, HTC, HME>::new(
            get_commands(),
            get_datatypes(),
            get_shortcuts(),
            prompt,
        );

        println!("Shell started (try ###)");

        Self {
            parser,
            _terminal: RawMode::new(0),
            is_shortcut,
            command_dispatcher,
            shortcut_dispatcher,
        }
    }

    pub fn run(&mut self) {
        let is_shortcut = self.is_shortcut;
        let command_dispatcher = self.command_dispatcher;
        let shortcut_dispatcher = self.shortcut_dispatcher;

        loop {
            let continue_running = self.parser.parse_input(move |input| {
                exec::<IML, ERRTYPE>(input, is_shortcut, command_dispatcher, shortcut_dispatcher)
            });

            if !continue_running {
                println!("Shell exited...");
                break;
            }
        }
    }
}

fn exec<const IML: usize, ERRTYPE: Debug>(
    input: &String<IML>,
    is_shortcut: fn(&str) -> bool,
    command_dispatcher: fn(&str) -> Result<(), ERRTYPE>,
    shortcut_dispatcher: fn(&str) -> Result<(), String<IML>>,
) {
    let result: Result<(), String<IML>> = if is_shortcut(input) {
        shortcut_dispatcher(input)
    } else {
        command_dispatcher(input).map_err(|e| {
            let mut err_str = String::<IML>::new();
            use core::fmt::Write;
            write!(&mut err_str, "{:?}", e).unwrap();
            err_str
        })
    };

    match result {
        Ok(_) => println!("Success: {}", input),
        Err(e) => println!("Error: {} for line '{}'", e, input),
    }
}
