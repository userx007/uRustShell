#![allow(clippy::unbuffered_bytes)]

/// InputParser is a generic, configurable command-line input handler designed for embedded or constrained environments. It supports:
/// - Autocompletion
/// - Input history
/// - Command parsing
/// - Special key handling (arrows, backspace, tab, etc.)
/// - Inline command help and shortcuts
///
/// It integrates with:
/// - Autocomplete
/// - History
/// - InputBuffer
/// - DisplayRenderer

use std::io::{self, Write};
use heapless::{Vec, String};

use crate::autocomplete::Autocomplete;
use crate::history::History;
use crate::input::buffer::InputBuffer;
use crate::input::renderer::DisplayRenderer;
use crate::input::key_reader::platform::read_key;
use crate::input::key_reader::Key;


/// # Type Parameters
/// - `NC`: Maximum number of autocomplete candidates.
/// - `FNL`: Maximum number of characters used for autocomplete matching.
/// - `IML`: Maximum input buffer length.
/// - `HTC`: History capacity (number of entries).
/// - `HME`: Maximum entry length in history.
///
/// # Fields
/// - `shell_commands`: Static list of available shell commands and their descriptions.
/// - `shell_datatypes`: Description of supported argument types.
/// - `shell_shortcuts`: Description of available keyboard shortcuts.
/// - `autocomplete`: Autocomplete engine for input suggestions.
/// - `history`: Command history manager (heap-allocated or stack-based depending on feature flags).
/// - `buffer`: Input buffer for editing and cursor movement (heap-allocated or stack-based depending on feature flags).
/// - `prompt`: Static prompt string displayed to the user.

pub struct InputParser<'a, const NC: usize, const FNL: usize, const IML: usize, const HTC: usize, const HME: usize> {
    shell_commands : &'static [(&'static str, &'static str)],
    shell_datatypes: &'static str,
    shell_shortcuts: &'static str,
    autocomplete: Autocomplete<'a, NC, FNL>,

    #[cfg(feature = "heap-history")]
    history: Box<History<HTC, HME>>,
    #[cfg(not(feature = "heap-history"))]
    history: History<HTC, HME>,

    #[cfg(feature = "heap-input-buffer")]
    buffer: Box<InputBuffer<IML>>,
    #[cfg(not(feature = "heap-input-buffer"))]
    buffer: InputBuffer<IML>,

    prompt: &'static str,
}

impl<'a, const NC: usize, const FNL: usize, const IML: usize, const HTC: usize, const HME: usize>
    InputParser<'a, NC, FNL, IML, HTC, HME>
{

    /// Creates a new instance of `InputParser` with the provided shell configuration and prompt.
    ///
    /// # Parameters
    /// - `shell_commands`: A static list of command names and their descriptions.
    /// - `shell_datatypes`: A static string describing supported argument types.
    /// - `shell_shortcuts`: A static string listing available keyboard shortcuts.
    /// - `prompt`: The prompt string displayed to the user during input.
    ///
    /// # Behavior
    /// - Initializes autocomplete candidates from the command names.
    /// - Constructs the history and input buffer, using heap or stack allocation depending on feature flags.

    pub fn new(
        shell_commands: &'static [(&'static str, &'static str)],
        shell_datatypes: &'static str,
        shell_shortcuts: &'static str,
        prompt: &'static str,
    ) -> Self {
        let mut candidates = Vec::<&'a str, NC>::new();
        for &(first, _) in shell_commands {
            candidates.push(first).unwrap();
        }

        #[cfg(feature = "heap-history")]
        let history = Box::new(History::<HTC, HME>::new());
        #[cfg(not(feature = "heap-history"))]
        let history = History::<HTC, HME>::new();

        #[cfg(feature = "heap-input-buffer")]
        let buffer = Box::new(InputBuffer::<IML>::new());
        #[cfg(not(feature = "heap-input-buffer"))]
        let buffer = InputBuffer::<IML>::new();


        Self {
            shell_commands,
            shell_datatypes,
            shell_shortcuts,
            autocomplete: Autocomplete::<'a, NC, FNL>::new(candidates),
            history,
            buffer,
            prompt,
        }
    }

    /// Handles a single character input from the user.
    ///
    /// If the character is successfully inserted into the input buffer:
    /// - Updates the autocomplete engine with the first FNL characters.
    /// - Retrieves the current autocomplete suggestion.
    /// - If the suggestion differs from the input prefix, overwrites the buffer with the suggestion.
    ///
    /// If the character cannot be inserted (e.g., buffer full):
    /// - Displays a boundary marker and flushes stdout.
    ///
    /// Finally, renders the updated buffer and prompt to the display.

    pub fn handle_char(&mut self, ch: char) {
        if self.buffer.insert(ch) {
            let input_full = self.buffer.to_string();
            let autocomplete_input: String<FNL> = input_full.chars().take(FNL).collect();
            self.autocomplete.update_input(autocomplete_input);
            let suggestion = self.autocomplete.current_input();
            let input_prefix: String<FNL> = input_full.chars().take(FNL).collect();
            if suggestion != input_prefix {
                let mut new_buf = String::<IML>::new();
                new_buf.push_str(suggestion).ok();
                for c in input_full.chars().skip(FNL) {
                    let _ = new_buf.push(c);
                }
                self.buffer.overwrite(&new_buf);
            }
        } else {
            DisplayRenderer::boundary_marker();
            let _ = io::stdout().flush();
        }
        let cursor_pos = self.buffer.cursor().min(self.buffer.len());
        DisplayRenderer::render(self.prompt, &self.buffer.to_string(), cursor_pos);
    }


    /// Handles the backspace key event within the input buffer.
    ///
    /// If a character is successfully removed from the buffer:
    /// - Converts the buffer to a string.
    /// - Extracts up to `FNL` characters from the input.
    /// - Updates the autocomplete system with the truncated input.
    ///
    /// If no character can be removed (e.g., buffer is empty), triggers a bell sound.
    ///
    /// Finally, re-renders the prompt and buffer display to reflect the current state.

    pub fn handle_backspace(&mut self) {
        if self.buffer.backspace() {
            let input_full = self.buffer.to_string();
            let mut input_fn = String::<FNL>::new();
            for c in input_full.chars().take(FNL) {
                let _ = input_fn.push(c);
            }
            self.autocomplete.update_input(input_fn);
        } else {
            DisplayRenderer::bell();
        }
        DisplayRenderer::render(self.prompt, &self.buffer.to_string(), self.buffer.cursor());
    }


    /// Handles the tab key event to cycle through autocomplete suggestions.
    ///
    /// If `reverse` is `true`, triggers reverse cycling (Shift+Tab); otherwise, cycles forward.
    ///
    /// Updates the input buffer with the current autocomplete suggestion:
    /// - Takes up to `FNL` characters from the suggestion.
    /// - Appends the remainder of the original input (after `FNL`).
    ///
    /// Overwrites the buffer with the new input and re-renders the prompt and buffer display.

    pub fn handle_tab(&mut self, reverse: bool) {
        if reverse {
            self.autocomplete.cycle_backward();
        } else {
            self.autocomplete.cycle_forward();
        }
        let suggestion = self.autocomplete.current_input();
        let input_full = self.buffer.to_string();
        let mut new_buf = String::<IML>::new();
        for c in suggestion.chars().take(FNL) {
            let _ = new_buf.push(c);
        }
        for c in input_full.chars().skip(FNL) {
            let _ = new_buf.push(c);
        }
        self.buffer.overwrite(&new_buf);
        DisplayRenderer::render(self.prompt, &self.buffer.to_string(), self.buffer.cursor());
    }


    /// Finalizes the input process by returning the current buffer content as a string.
    ///
    /// Converts the internal buffer to a `String<IML>` and returns it without modification.

    pub fn finalize(&mut self) -> String<IML> {
        self.buffer.to_string()
    }


    /// Displays a formatted list of available shell commands.
    ///
    /// Prints each command name and its specification, aligned for readability.
    /// Calculates the maximum command name length to ensure consistent formatting.

    pub fn list_commands(&self)  {
        println!("\r\n⚡ Commands:");
        let max_name_len = self.shell_commands.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
        for (name, spec) in self.shell_commands {
            println!("{:>width$} : {}", name, spec, width = max_name_len);
        }
    }


    /// Displays all available shell commands, shortcuts, and argument types.
    ///
    /// - Calls `list_commands()` to print the command list.
    /// - Prints predefined shell shortcuts.

    fn list_all(&self) {
        self.list_commands();
        print!("\n⚡ Shortcuts:\n### : list all\n##  : list cmds\n#q  : exit\n#h  : list history\n#c  : clear history\n#N  : exec from history at index N\n");
        print!("\n⚡ User shortcuts:\n{}\n", self.shell_shortcuts);
        print!("\n📝 Arg types:\n{}\n", self.shell_datatypes);
    }


    /// Handles special hashtag-prefixed input commands.
    ///
    /// Supports the following commands:
    /// - `"q"`: Quits or exits the current context (returns `false`, no output).
    /// - `"#"`: Displays available commands via `list_all()`.
    /// - `"h"`: Shows command history.
    /// - `"c"`: Clears command history.
    /// - Numeric input: Attempts to retrieve a history entry by index.
    ///
    /// Returns a tuple:
    /// - `bool`: Indicates whether the command was handled.

    fn handle_hashtag(&mut self, input: &str) -> (bool, Option<String<IML>>) {
        match input {
            "q" => (false, None),
            "#" => {
                self.list_commands();
                (true, None)
            }
            "##" => {
                self.list_all();
                (true, None)
            }
            "h" => {
                self.history.show::<IML>();
                (true, None)
            }
            "c" => {
                self.history.clear();
                println!("🧹 History cleared");
                (true, None)
            }
            _ => {
                if let Ok(index) = input.parse::<usize>() {
                    if let Some(entry) = self.history.get(index) {
                        return (true, Some(entry));
                    } else {
                        println!("⚠️ No history entry at index {}", index);
                    }
                } else {
                    println!("🚫 Not implemented");
                }
                (true, None)
            }
        }
    }


    /// Parses user input from `stdin` and handles interactive editing and command execution.
    ///
    /// Supports various key bindings for editing and navigation:
    /// - `Enter`: Finalizes input.
    /// - `Backspace`: Deletes character before cursor.
    /// - `Tab` / `Shift+Tab`: Cycles autocomplete suggestions.
    /// - `Ctrl+U`: Deletes from cursor to start of line.
    /// - `Ctrl+K`: Deletes from cursor to end of line.
    /// - `Ctrl+D`: Clears the entire buffer.
    /// - Arrow keys: Navigates through buffer or command history.
    /// - `Home` / `End`: Moves cursor to start/end of line.
    /// - `Delete`: Deletes character at cursor.
    ///
    /// After input is finalized:
    /// - If input starts with `#`, it is treated as a special command (e.g., history or help).
    /// - Otherwise, the input is executed via the provided `exec` callback and stored in history.
    ///
    /// Returns `true` if input was successfully handled or executed, `false` if the user requested to quit.

    pub fn parse_input<F>(&mut self, exec: F) -> bool
    where
        F: Fn(&String<IML>),
    {
        DisplayRenderer::render(self.prompt, "", 0);

        loop {
            let key = match read_key() {
                Ok(k) => k,
                Err(_) => continue,
            };

            match key {
                Key::Enter => {
                    println!();
                    break;
                }

                Key::Backspace => {
                    self.handle_backspace();
                }

                Key::Tab => {
                    self.handle_tab(false);
                }

                Key::ShiftTab => {
                    self.handle_tab(true);
                }

                Key::CtrlU => {
                    self.buffer.delete_to_start();
                    DisplayRenderer::render(self.prompt, &self.buffer.to_string(), self.buffer.cursor());
                }

                Key::CtrlK => {
                    self.buffer.delete_to_end();
                    DisplayRenderer::render(self.prompt, &self.buffer.to_string(), self.buffer.cursor());
                }

                Key::CtrlD => {
                    self.buffer.clear();
                    DisplayRenderer::render(self.prompt, "", 0);
                }

                Key::ArrowLeft => {
                    self.buffer.move_left();
                    DisplayRenderer::render(self.prompt, &self.buffer.to_string(), self.buffer.cursor());
                }

                Key::ArrowRight => {
                    self.buffer.move_right();
                    DisplayRenderer::render(self.prompt, &self.buffer.to_string(), self.buffer.cursor());
                }

                Key::ArrowUp => {
                    if let Some(cmd) = self.history.get_next_entry::<IML>() {
                        self.buffer.overwrite(&cmd);
                        DisplayRenderer::render(self.prompt, &self.buffer.to_string(), self.buffer.cursor());
                    }
                }

                Key::ArrowDown => {
                    if let Some(cmd) = self.history.get_prev_entry::<IML>() {
                        self.buffer.overwrite(&cmd);
                        DisplayRenderer::render(self.prompt, &self.buffer.to_string(), self.buffer.cursor());
                    }
                }

                Key::Home => {
                    self.buffer.move_home();
                    DisplayRenderer::render(self.prompt, &self.buffer.to_string(), self.buffer.cursor());
                }

                Key::End => {
                    self.buffer.move_end();
                    DisplayRenderer::render(self.prompt, &self.buffer.to_string(), self.buffer.cursor());
                }

                Key::Delete => {
                    self.buffer.delete_at_cursor();
                    DisplayRenderer::render(self.prompt, &self.buffer.to_string(), self.buffer.cursor());
                }

                Key::PageUp => {
                    if let Some(cmd) = self.history.get_first_entry::<IML>() {
                        self.buffer.overwrite(&cmd);
                        DisplayRenderer::render(self.prompt, &self.buffer.to_string(), self.buffer.cursor());
                    }
                }

                Key::PageDown => {
                    if let Some(cmd) = self.history.get_last_entry::<IML>() {
                        self.buffer.overwrite(&cmd);
                        DisplayRenderer::render(self.prompt, &self.buffer.to_string(), self.buffer.cursor());
                    }
                }

                Key::Char(c) => {
                    if Self::valid_byte(c as u8) {
                        self.handle_char(c);
                    }
                }

                _ => {}
            }
        }

        // Finalize input
        let mut retval = true;
        let final_input = self.finalize();

        if !final_input.is_empty() {
            if let Some(stripped) = final_input.strip_prefix('#') {
                let (new_retval, maybe_history_command) = self.handle_hashtag(stripped);
                retval = new_retval;
                if let Some(history_command) = maybe_history_command {
                    exec(&history_command);
                }
            } else {
                exec(&final_input);
                self.history.push(&final_input);
            }

            self.buffer.clear();
        }

        retval
    }

    /// Checks whether a given byte represents a valid ASCII character for input.
    ///
    /// A byte is considered valid if:
    /// - It is an ASCII character.
    /// - It is alphanumeric, a space, or falls within the printable ASCII range (`'!'` to `'~'`).
    ///
    /// Returns `true` if the byte is valid for input; otherwise, returns `false`.

    fn valid_byte(b: u8) -> bool {
        let c = b as char;
        c.is_ascii() && (c.is_ascii_alphanumeric() || c == ' ' || matches!(c, '!'..='~'))
    }
}


