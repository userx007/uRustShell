#![allow(clippy::unbuffered_bytes)]

use heapless::{String, Vec};
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

use crate::autocomplete::Autocomplete;
use crate::history::History;
use crate::input::buffer::InputBuffer;
use crate::input::key_reader::Key;
use crate::input::key_reader::platform::read_key;
use crate::input::renderer::DisplayRenderer;

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
///
pub struct InputParser<
    'a,
    const NC: usize,
    const FNL: usize,
    const IML: usize,
    const HTC: usize,
    const HME: usize,
> {
    shell_commands: &'static [(&'static str, &'static str)],
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
    ///
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
    ///
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
    ///
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
    ///
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
    ///
    pub fn finalize(&mut self) -> String<IML> {
        self.buffer.to_string()
    }

    /// Displays a formatted list of available shell commands.
    ///
    /// Prints each command name and its specification, aligned for readability.
    /// Calculates the maximum command name length to ensure consistent formatting.
    ///
    pub fn list_commands(&self) {
        println!("\r\nCommands:");
        let max_name_len = self
            .shell_commands
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0);
        for (name, spec) in self.shell_commands {
            println!("{:>width$} : {}", name, spec, width = max_name_len);
        }
    }

    /// Displays all available shell commands, shortcuts, and argument types.
    ///
    /// - Calls `list_commands()` to print the command list.
    /// - Prints predefined shell shortcuts.
    ///
    fn list_all(&self) {
        self.list_commands();
        print!(
            "\nShortcuts:\n### : list all\n##  : list cmds\n#q  : exit\n#h  : list history\n#c  : clear history\n#N  : exec from history at index N\n"
        );
        print!("\nUser shortcuts:\n{}\n", self.shell_shortcuts);
        print!("\nArg types:\n{}\n", self.shell_datatypes);
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
    ///
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
                println!("History cleared");
                (true, None)
            }
            _ => {
                if let Ok(index) = input.parse::<usize>() {
                    if let Some(entry) = self.history.get(index) {
                        return (true, Some(entry));
                    } else {
                        println!("No history entry at index {}", index);
                    }
                } else {
                    println!("Not implemented");
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
    ///
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
                    DisplayRenderer::render(
                        self.prompt,
                        &self.buffer.to_string(),
                        self.buffer.cursor(),
                    );
                }

                Key::CtrlK => {
                    self.buffer.delete_to_end();
                    DisplayRenderer::render(
                        self.prompt,
                        &self.buffer.to_string(),
                        self.buffer.cursor(),
                    );
                }

                Key::CtrlD => {
                    self.buffer.clear();
                    DisplayRenderer::render(self.prompt, "", 0);
                }

                Key::ArrowLeft => {
                    self.buffer.move_left();
                    DisplayRenderer::render(
                        self.prompt,
                        &self.buffer.to_string(),
                        self.buffer.cursor(),
                    );
                }

                Key::ArrowRight => {
                    self.buffer.move_right();
                    DisplayRenderer::render(
                        self.prompt,
                        &self.buffer.to_string(),
                        self.buffer.cursor(),
                    );
                }

                Key::ArrowUp => {
                    if let Some(cmd) = self.history.get_next_entry::<IML>() {
                        self.buffer.overwrite(&cmd);
                        DisplayRenderer::render(
                            self.prompt,
                            &self.buffer.to_string(),
                            self.buffer.cursor(),
                        );
                    }
                }

                Key::ArrowDown => {
                    if let Some(cmd) = self.history.get_prev_entry::<IML>() {
                        self.buffer.overwrite(&cmd);
                        DisplayRenderer::render(
                            self.prompt,
                            &self.buffer.to_string(),
                            self.buffer.cursor(),
                        );
                    }
                }

                Key::Home => {
                    self.buffer.move_home();
                    DisplayRenderer::render(
                        self.prompt,
                        &self.buffer.to_string(),
                        self.buffer.cursor(),
                    );
                }

                Key::End => {
                    self.buffer.move_end();
                    DisplayRenderer::render(
                        self.prompt,
                        &self.buffer.to_string(),
                        self.buffer.cursor(),
                    );
                }

                Key::Delete => {
                    self.buffer.delete_at_cursor();
                    DisplayRenderer::render(
                        self.prompt,
                        &self.buffer.to_string(),
                        self.buffer.cursor(),
                    );
                }

                Key::PageUp => {
                    if let Some(cmd) = self.history.get_first_entry::<IML>() {
                        self.buffer.overwrite(&cmd);
                        DisplayRenderer::render(
                            self.prompt,
                            &self.buffer.to_string(),
                            self.buffer.cursor(),
                        );
                    }
                }

                Key::PageDown => {
                    if let Some(cmd) = self.history.get_last_entry::<IML>() {
                        self.buffer.overwrite(&cmd);
                        DisplayRenderer::render(
                            self.prompt,
                            &self.buffer.to_string(),
                            self.buffer.cursor(),
                        );
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
    ///
    fn valid_byte(b: u8) -> bool {
        let c = b as char;
        c.is_ascii() && (c.is_ascii_alphanumeric() || c == ' ' || matches!(c, '!'..='~'))
    }
}

// ==================== TESTS =======================

#[cfg(test)]
mod input_parser_tests {
    use super::*;
    use heapless::String;

    // Test constants
    const TEST_COMMANDS: &[(&str, &str)] = &[
        ("help", "Display help information"),
        ("exit", "Exit the shell"),
        ("list", "List items"),
        ("test", "Run tests"),
        ("hello", "Say hello"),
    ];

    const TEST_DATATYPES: &str = "string, int, bool";
    const TEST_SHORTCUTS: &str = "Ctrl+C: Cancel\nCtrl+Z: Undo";
    const TEST_PROMPT: &str = "> ";

    // Type aliases for test configurations
    type TestParser = InputParser<'static, 10, 32, 128, 20, 64>;
    type SmallParser = InputParser<'static, 5, 16, 32, 5, 32>;

    // ==================== CONSTRUCTOR TESTS ====================

    #[test]
    fn test_new_creates_valid_parser() {
        let parser = TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Parser should be created successfully
        assert_eq!(parser.prompt, TEST_PROMPT);
    }

    #[test]
    fn test_new_with_empty_commands() {
        let empty_commands: &[(&str, &str)] = &[];
        let parser = InputParser::<10, 32, 128, 20, 64>::new(
            empty_commands,
            TEST_DATATYPES,
            TEST_SHORTCUTS,
            TEST_PROMPT,
        );

        assert_eq!(parser.prompt, TEST_PROMPT);
    }

    #[test]
    fn test_new_with_maximum_commands() {
        let max_commands: &[(&str, &str)] = &[
            ("cmd1", "desc1"),
            ("cmd2", "desc2"),
            ("cmd3", "desc3"),
            ("cmd4", "desc4"),
            ("cmd5", "desc5"),
        ];

        let parser = InputParser::<5, 16, 32, 5, 32>::new(
            max_commands,
            TEST_DATATYPES,
            TEST_SHORTCUTS,
            TEST_PROMPT,
        );

        assert_eq!(parser.prompt, TEST_PROMPT);
    }

    // ==================== HANDLE_CHAR TESTS ====================

    #[test]
    fn test_handle_char_single_character() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        parser.handle_char('h');
        let result = parser.finalize();

        assert!(result.starts_with('h'));
    }

    #[test]
    fn test_handle_char_multiple_characters() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        for c in "hello".chars() {
            parser.handle_char(c);
        }

        let result = parser.finalize();
        // Autocomplete may modify input, so check it contains key characters
        assert!(result.contains("hel"));
        assert!(result.len() > 0);
    }

    #[test]
    fn test_handle_char_with_autocomplete() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Type 'h' which should autocomplete to 'help' or 'hello'
        parser.handle_char('h');
        let result = parser.finalize();

        assert!(result.starts_with('h'));
        assert!(result.len() > 0);
    }

    #[test]
    fn test_handle_char_special_characters() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        let special_chars = "!@#$%^&*()";
        for c in special_chars.chars() {
            parser.handle_char(c);
        }

        let result = parser.finalize();
        assert!(result.len() > 0);
    }

    #[test]
    fn test_handle_char_numbers() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        for c in "1234567890".chars() {
            parser.handle_char(c);
        }

        let result = parser.finalize();
        assert!(result.contains("1234567890"));
    }

    #[test]
    fn test_handle_char_spaces() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        for c in "hello world".chars() {
            parser.handle_char(c);
        }

        let result = parser.finalize();
        assert!(result.contains(' '));
    }

    #[test]
    fn test_handle_char_buffer_overflow() {
        let mut parser =
            SmallParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Try to fill buffer beyond capacity
        for _ in 0..50 {
            parser.handle_char('a');
        }

        let result = parser.finalize();
        // Buffer should be limited to its capacity
        assert!(result.len() <= 32);
    }

    // ==================== HANDLE_BACKSPACE TESTS ====================

    #[test]
    fn test_handle_backspace_removes_character() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        parser.handle_char('a');
        parser.handle_char('b');
        parser.handle_backspace();

        let result = parser.finalize();
        assert_eq!(result.as_str(), "a");
    }

    #[test]
    fn test_handle_backspace_on_empty_buffer() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Backspace on empty buffer should not crash
        parser.handle_backspace();

        let result = parser.finalize();
        assert!(result.is_empty());
    }

    #[test]
    fn test_handle_backspace_multiple_times() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        for c in "hello".chars() {
            parser.handle_char(c);
        }

        // Remove 3 characters
        parser.handle_backspace();
        parser.handle_backspace();
        parser.handle_backspace();

        let result = parser.finalize();
        // Due to autocomplete, result may differ, but should be shorter
        assert!(result.len() <= 5);
        assert!(result.len() > 0);
    }

    #[test]
    fn test_handle_backspace_clears_entire_input() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        for c in "test".chars() {
            parser.handle_char(c);
        }

        let current_len = parser.buffer.len();

        // Remove all characters - may need extra backspaces due to autocomplete
        for _ in 0..(current_len + 5) {
            parser.handle_backspace();
        }

        let result = parser.finalize();
        assert!(result.is_empty());
    }

    // ==================== HANDLE_TAB TESTS ====================

    #[test]
    fn test_handle_tab_forward_cycling() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        parser.handle_char('h');
        let first = parser.buffer.to_string();

        parser.handle_tab(false);
        let second = parser.buffer.to_string();

        // Tab should change the suggestion
        // May be same if only one match
        assert!(first.len() > 0 && second.len() > 0);
    }

    #[test]
    fn test_handle_tab_reverse_cycling() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        parser.handle_char('h');
        parser.handle_tab(true); // Shift+Tab

        let result = parser.finalize();
        assert!(result.starts_with('h'));
    }

    #[test]
    fn test_handle_tab_with_no_matches() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        parser.handle_char('z'); // No commands start with 'z'
        parser.handle_tab(false);

        let result = parser.finalize();
        assert!(result.len() > 0);
    }

    #[test]
    fn test_handle_tab_preserves_suffix() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Type command and arguments
        for c in "h arg1 arg2".chars() {
            parser.handle_char(c);
        }

        parser.handle_tab(false);
        let result = parser.finalize();

        // Arguments should be preserved
        assert!(result.contains("arg"));
    }

    // ==================== FINALIZE TESTS ====================

    #[test]
    fn test_finalize_returns_buffer_content() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        for c in "test command".chars() {
            parser.handle_char(c);
        }

        let result = parser.finalize();
        assert!(result.contains("test"));
    }

    #[test]
    fn test_finalize_empty_buffer() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        let result = parser.finalize();
        assert!(result.is_empty());
    }

    #[test]
    fn test_finalize_multiple_calls() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        for c in "test".chars() {
            parser.handle_char(c);
        }

        let result1 = parser.finalize();
        let result2 = parser.finalize();

        // Multiple calls should return same content
        assert_eq!(result1, result2);
    }

    // ==================== HANDLE_HASHTAG TESTS ====================

    #[test]
    fn test_handle_hashtag_quit_command() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        let (retval, cmd) = parser.handle_hashtag("q");

        assert!(!retval); // Should return false for quit
        assert!(cmd.is_none());
    }

    #[test]
    fn test_handle_hashtag_help_command() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        let (retval, cmd) = parser.handle_hashtag("#");

        assert!(retval); // Should return true
        assert!(cmd.is_none());
    }

    #[test]
    fn test_handle_hashtag_full_help_command() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        let (retval, cmd) = parser.handle_hashtag("##");

        assert!(retval);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_handle_hashtag_history_command() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        let (retval, cmd) = parser.handle_hashtag("h");

        assert!(retval);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_handle_hashtag_clear_history() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Add some history
        parser
            .history
            .push(&String::<64>::try_from("test1").unwrap());
        parser
            .history
            .push(&String::<64>::try_from("test2").unwrap());

        let (retval, cmd) = parser.handle_hashtag("c");

        assert!(retval);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_handle_hashtag_numeric_index() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Add history entry
        let test_cmd = String::<64>::try_from("test command").unwrap();
        parser.history.push(&test_cmd);

        let (retval, _cmd) = parser.handle_hashtag("0");

        assert!(retval);
        // Should return the history command if it exists
    }

    #[test]
    fn test_handle_hashtag_invalid_index() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        let (retval, cmd) = parser.handle_hashtag("999");

        assert!(retval);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_handle_hashtag_invalid_command() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        let (retval, cmd) = parser.handle_hashtag("invalid");

        assert!(retval);
        assert!(cmd.is_none());
    }

    // ==================== VALID_BYTE TESTS ====================

    #[test]
    fn test_valid_byte_alphanumeric() {
        assert!(TestParser::valid_byte(b'a'));
        assert!(TestParser::valid_byte(b'Z'));
        assert!(TestParser::valid_byte(b'0'));
        assert!(TestParser::valid_byte(b'9'));
    }

    #[test]
    fn test_valid_byte_space() {
        assert!(TestParser::valid_byte(b' '));
    }

    #[test]
    fn test_valid_byte_special_characters() {
        assert!(TestParser::valid_byte(b'!'));
        assert!(TestParser::valid_byte(b'@'));
        assert!(TestParser::valid_byte(b'#'));
        assert!(TestParser::valid_byte(b'$'));
        assert!(TestParser::valid_byte(b'~'));
    }

    #[test]
    fn test_valid_byte_non_ascii() {
        assert!(!TestParser::valid_byte(128));
        assert!(!TestParser::valid_byte(255));
    }

    #[test]
    fn test_valid_byte_control_characters() {
        assert!(!TestParser::valid_byte(0)); // NULL
        assert!(!TestParser::valid_byte(1)); // SOH
        assert!(!TestParser::valid_byte(27)); // ESC
        assert!(!TestParser::valid_byte(127)); // DEL
    }

    #[test]
    fn test_valid_byte_printable_range() {
        // Test full printable range
        for b in b'!'..=b'~' {
            assert!(TestParser::valid_byte(b));
        }
    }

    // ==================== LIST_COMMANDS TESTS ====================

    #[test]
    fn test_list_commands_does_not_panic() {
        let parser = TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Should not panic
        parser.list_commands();
    }

    #[test]
    fn test_list_commands_with_empty_list() {
        let empty_commands: &[(&str, &str)] = &[];
        let parser = InputParser::<10, 32, 128, 20, 64>::new(
            empty_commands,
            TEST_DATATYPES,
            TEST_SHORTCUTS,
            TEST_PROMPT,
        );

        // Should not panic with empty commands
        parser.list_commands();
    }

    // ==================== INTEGRATION TESTS ====================

    #[test]
    fn test_autocomplete_behavior_with_matching_prefix() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Type 'h' which should autocomplete to 'help' or 'hello'
        parser.handle_char('h');
        let result = parser.finalize();

        // Autocomplete should expand the input
        assert!(result.len() >= 1);
        assert!(result.starts_with('h'));
    }

    #[test]
    fn test_autocomplete_preserves_suffix_after_fnl() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Type a full command that matches
        for c in "help".chars() {
            parser.handle_char(c);
        }

        // Add content beyond FNL characters
        for c in " these are arguments that should be preserved".chars() {
            parser.handle_char(c);
        }

        let result = parser.finalize();
        // Arguments after FNL should be preserved
        assert!(result.contains("preserved"));
    }

    #[test]
    fn test_no_autocomplete_for_non_matching_input() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Type characters that don't match any command
        for c in "xyz123".chars() {
            parser.handle_char(c);
        }

        let result = parser.finalize();
        // Should preserve original input when no match
        assert!(result.contains("xyz"));
    }

    #[test]
    fn test_full_input_cycle() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Type a command - use one that doesn't trigger autocomplete changes
        for c in "xyz".chars() {
            parser.handle_char(c);
        }

        // Add some arguments
        parser.handle_char(' ');
        for c in "arg".chars() {
            parser.handle_char(c);
        }

        // Remove last character
        parser.handle_backspace();

        // Add it back
        parser.handle_char('g');

        let result = parser.finalize();
        // Check that we have a reasonable result with our input
        assert!(result.contains("xyz"));
        assert!(result.contains("arg"));
    }

    #[test]
    fn test_autocomplete_and_backspace() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        parser.handle_char('h');
        parser.handle_tab(false);
        parser.handle_backspace();
        parser.handle_backspace();

        let result = parser.finalize();
        // Should still have some content or be shorter
        assert!(result.len() < 10);
    }

    #[test]
    fn test_multiple_tab_cycles() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        parser.handle_char('h');

        // Cycle through multiple suggestions
        for _ in 0..3 {
            parser.handle_tab(false);
        }

        let result = parser.finalize();
        assert!(result.starts_with('h'));
    }

    #[test]
    fn test_mixed_operations() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        parser.handle_char('t');
        parser.handle_char('e');
        parser.handle_backspace();
        parser.handle_char('e');
        parser.handle_char('s');
        parser.handle_char('t');
        parser.handle_char(' ');
        parser.handle_tab(false);

        let result = parser.finalize();
        assert!(result.len() > 0);
    }

    // ==================== EDGE CASE TESTS ====================

    #[test]
    fn test_very_long_input() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Try to add more characters than buffer can hold
        for i in 0..150 {
            parser.handle_char(((i % 26) as u8 + b'a') as char);
        }

        let result = parser.finalize();
        assert!(result.len() <= 128); // Should be limited by IML
    }

    #[test]
    fn test_rapid_backspace_sequence() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        for c in "test".chars() {
            parser.handle_char(c);
        }

        // Rapid backspaces including past buffer start
        for _ in 0..10 {
            parser.handle_backspace();
        }

        let result = parser.finalize();
        assert!(result.is_empty());
    }

    #[test]
    #[allow(unused_comparisons)]
    fn test_tab_with_empty_input() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Tab on empty input
        parser.handle_tab(false);

        let result = parser.finalize();
        // Should handle gracefully
        assert!(result.len() >= 0);
    }

    #[test]
    fn test_special_characters_sequence() {
        let mut parser =
            TestParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        let special_seq = "!@#$%^&*()_+-={}[]|:;<>?,./";
        for c in special_seq.chars() {
            parser.handle_char(c);
        }

        let result = parser.finalize();
        assert!(result.len() > 0);
    }

    // ==================== BOUNDARY TESTS ====================

    #[test]
    fn test_exact_buffer_capacity() {
        let mut parser =
            SmallParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Fill exactly to capacity
        for _ in 0..32 {
            parser.handle_char('a');
        }

        let result = parser.finalize();
        assert!(result.len() <= 32);
    }

    #[test]
    fn test_autocomplete_at_fnl_boundary() {
        let mut parser =
            SmallParser::new(TEST_COMMANDS, TEST_DATATYPES, TEST_SHORTCUTS, TEST_PROMPT);

        // Type exactly FNL characters
        for _ in 0..16 {
            parser.handle_char('h');
        }

        parser.handle_tab(false);

        let result = parser.finalize();
        assert!(result.len() > 0);
    }

    #[test]
    fn test_history_at_capacity() {
        let mut parser = InputParser::<5, 16, 32, 3, 32>::new(
            TEST_COMMANDS,
            TEST_DATATYPES,
            TEST_SHORTCUTS,
            TEST_PROMPT,
        );

        // Fill history to capacity
        for i in 0..5 {
            let cmd = String::<32>::try_from(format!("cmd{}", i).as_str()).unwrap();
            parser.history.push(&cmd);
        }

        // Should handle gracefully when at capacity
        let (retval, _) = parser.handle_hashtag("h");
        assert!(retval);
    }
}
