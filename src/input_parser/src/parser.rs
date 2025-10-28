use std::io::{self, Read, Write};

// Import autocomplete and history modules
use crate::autocomplete::Autocomplete;
use crate::history::{History, MAX_LEN};
use crate::raw_mode::RawMode;

/// Handles user input parsing, autocomplete, and history navigation.
pub struct InputParser<'a> {

    /// Commands specifications
    commands_spec: &'static [(&'static str, &'static str)],

    /// Types info string
    types_info : &'static str,

    /// Shortcuts info string
    shortcuts_info : &'static str,

    /// Autocomplete engine for suggesting completions based on input
    autocomplete: Autocomplete<'a>,

    /// Command history manager
    history: History,

    /// Enables raw mode for terminal input (disables line buffering, etc.)
    _raw_mode: RawMode,
}

impl<'a> InputParser<'a> {

    /// Creates a new InputParser
    pub fn new(commands: &'static [(&'static str, &'static str)], types_info : &'static str, shortcuts_info : &'static str) -> Self {
        let candidates : Vec<&'a str> = commands.iter().map(|(name, _)| *name).collect();
        Self {
            types_info : types_info,
            shortcuts_info : shortcuts_info,
            commands_spec : commands,
            autocomplete: Autocomplete::new(candidates),
            history: History::new(),
            _raw_mode: RawMode::new(0), // Enables raw mode on stdin
        }
    }

    /// Parses user input from the terminal, supports editing, autocomplete, and history
    pub fn parse_input(&mut self) -> Option<String> {
        // Buffer to store input characters
        let mut buffer = ['\0'; MAX_LEN];

        // Cursor position within the buffer
        let mut cursor_pos = 0;

        // Current length of the input
        let mut length = 0;

        print!("\n> ");
        io::stdout().flush().unwrap();

        let mut bytes = io::stdin().bytes();

        // Main input loop
        while let Some(Ok(b)) = bytes.next() {
            match b {
                b'\n' => { // Enter key
                    if 0 != cursor_pos {
                        print!("\n")
                    }
                    break;
                }

                127 => { // Backspace
                    if cursor_pos > 0 {
                        for i in cursor_pos..length {
                            buffer[i - 1] = buffer[i];
                        }
                        length -= 1;
                        cursor_pos -= 1;
                        buffer[length] = '\0';

                        let input: String = buffer.iter().take(length).collect();
                        self.autocomplete.update_input(input.clone());

                        print!("\r> {}\x1b[K", self.autocomplete.current_input());
                        io::stdout().flush().unwrap();
                    } else {
                        print!("\x07"); // Bell sound
                    }
                }

                27 => { // Escape sequences (arrow keys, Home, End, etc.)
                    let b1 = bytes.next().unwrap().unwrap();
                    let b2 = bytes.next().unwrap().unwrap();
                    match (b1, b2) {
                        (91, 68) => { // Left arrow
                            if cursor_pos > 0 {
                                cursor_pos -= 1;
                            }
                        }
                        (91, 67) => { // Right arrow
                            if cursor_pos < length {
                                cursor_pos += 1;
                            }
                        }
                        (91, 51) => { // Delete
                            let _tilde = bytes.next();
                            if cursor_pos < length {
                                for i in cursor_pos..length - 1 {
                                    buffer[i] = buffer[i + 1];
                                }
                                buffer[length - 1] = '\0';
                                length -= 1;
                            }
                        }
                        (91, 72) | (91, 49) => { // Home
                            cursor_pos = 0;
                            if b2 == 49 {
                                let _tilde = bytes.next();
                            }
                        }
                        (91, 70) | (91, 52) => { // End
                            cursor_pos = length;
                            if b2 == 52 {
                                let _tilde = bytes.next();
                            }
                        }
                        (91, 65) => { // Up arrow (previous history)
                            if let Some(cmd) = self.history.previous() {
                                length = cmd.len().min(MAX_LEN);
                                cursor_pos = length;
                                buffer = ['\0'; MAX_LEN];
                                for (i, c) in cmd.chars().take(MAX_LEN).enumerate() {
                                    buffer[i] = c;
                                }
                            }
                        }
                        (91, 66) => { // Down arrow (next history)
                            if let Some(cmd) = self.history.next() {
                                length = cmd.len().min(MAX_LEN);
                                cursor_pos = length;
                                buffer = ['\0'; MAX_LEN];
                                for (i, c) in cmd.chars().take(MAX_LEN).enumerate() {
                                    buffer[i] = c;
                                }
                            } else {
                                length = 0;
                                cursor_pos = 0;
                                buffer = ['\0'; MAX_LEN];
                            }
                        }
                        (91, 90) => { // Shift-Tab: autocomplete in reverse
                            self.autocomplete.handle_shift_tab();
                            self.autocomplete_common(&mut buffer, &mut cursor_pos, &mut length);
                        }
                        _ => {}
                    }
                }
                21 => { // Ctrl+U: delete from start to cursor
                    let shift = length - cursor_pos;
                    for i in 0..shift {
                        buffer[i] = buffer[cursor_pos + i];
                    }
                    for i in shift..MAX_LEN {
                        buffer[i] = '\0';
                    }
                    length = shift;
                    cursor_pos = 0;
                }
                11 => { // Ctrl+K: delete from cursor to end
                    for i in cursor_pos..length {
                        buffer[i] = '\0';
                    }
                    length = cursor_pos;
                }
                4 => { // Ctrl+D: delete entire line
                    for i in 0..length {
                        buffer[i] = '\0';
                    }
                    length = 0;
                    cursor_pos = 0;
                }
                9 => { // Tab: autocomplete
                    self.autocomplete.handle_tab();
                    self.autocomplete_common(&mut buffer, &mut cursor_pos, &mut length);
                }
                b => { // Regular character input
                    if length < MAX_LEN {
                        for i in (cursor_pos..length).rev() {
                            buffer[i + 1] = buffer[i];
                        }
                        buffer[cursor_pos] = b as char;
                        length += 1;

                        let input: String = buffer.iter().take(length).collect();
                        self.autocomplete.update_input(input.clone());

                        let updated = self.autocomplete.current_input();
                        buffer = ['\0'; MAX_LEN];
                        for (i, c) in updated.chars().take(MAX_LEN).enumerate() {
                            buffer[i] = c;
                        }
                        length = updated.len().min(MAX_LEN);
                        cursor_pos = length;

                        print!("\r> {}\x1b[K", updated);
                        io::stdout().flush().unwrap();
                    } else {
                        print!("\x07"); // Bell sound
                    }
                }
            }

            // Refresh display
            let display: String = buffer.iter().take(length).collect();
            print!("\r\x1B[K> {}", display);
            print!("\x1B[{}G", cursor_pos + 3);
            io::stdout().flush().unwrap();
        }

        // Final input string
        let final_input: String = buffer.iter().take(length).collect();

        match &final_input[..] {
            "#q" => return None, // Exit condition
            "##" => {
                self.list_elements();
                return Some("".to_string());
            }
            "#h" => {
                self.history.list_with_indexes();
                return Some("".to_string());
            }
            _ if final_input.starts_with('#') => {
                if let Some(index_str) = final_input.strip_prefix('#') {
                    if let Ok(index) = index_str.parse::<usize>() {
                        if let Some(entry) = self.history.get_by_index(index) {
                            return Some(entry.clone());
                        } else {
                            print!("⚠️ No history entry at index {}", index);
                            return Some("".to_string());
                        }
                    }
                }
                print!("🚫 Invalid history command format");
                return Some("".to_string());
            }
            _ => {
                if !final_input.is_empty() {
                    self.history.push(final_input.clone());
                }
                Some(final_input)
            }
        }
    }

    fn autocomplete_common(&self, buffer : &mut [char; MAX_LEN], cursor_pos : &mut usize, length : &mut usize) {
        let updated = self.autocomplete.current_input();
        *buffer = ['\0'; MAX_LEN];
        for (i, c) in updated.chars().take(MAX_LEN).enumerate() {
            buffer[i] = c;
        }
        *length = updated.len().min(MAX_LEN);
        *cursor_pos = *length;

        print!("\r> {}\x1b[K", updated);
        io::stdout().flush().unwrap();
    }

    fn list_elements(&self) {
        let max_name_len = self.commands_spec.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
        print!("\r\n📌 Commands:\n");
        for (name, spec) in self.commands_spec {
            print!("{:>width$} : {}\n", name, spec, width = max_name_len);
        }
        print!("\n📌 Shortcuts:\n{}\n", self.shortcuts_info);
        print!("\n📌 Arg types:\n{}", self.types_info);
    }

}