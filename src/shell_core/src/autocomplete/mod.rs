
use heapless::{Vec, String};

/// Autocomplete struct for managing and filtering command candidates.
/// - `'a`: Lifetime for string slices.
/// - `NC`: Maximum number of candidates, NUM_COMMANDS.
/// - `FNL`: Maximum function name length, MAX_FUNCTION_NAME_LEN.
pub struct Autocomplete<'a, const NC: usize, const FNL: usize> {
    /// All possible candidates for autocompletion.
    candidates: Vec<&'a str, NC>,
    /// Filtered candidates matching the current input.
    filtered: Vec<&'a str, NC>,
    /// Current user input.
    input: String<FNL>,
    /// Index for cycling through filtered candidates with Tab.
    tab_index: usize,
}

impl<'a, const NC: usize, const FNL: usize> Autocomplete<'a, NC, FNL> {
    /// Creates a new Autocomplete instance with the given candidates.
    pub fn new(candidates: Vec<&'a str, NC>) -> Self {
        Self {
            candidates,
            filtered: Vec::new(),
            input: String::new(),
            tab_index: 0,
        }
    }

    /// Updates the input string and filters candidates accordingly.
    /// - If only one match, auto-completes input.
    /// - If multiple matches, fills input with the longest common prefix.
    pub fn update_input(&mut self, new_input: String<FNL>) {
        self.input = new_input;
        self.filtered.clear();
        for c in self.candidates.iter().copied() {
            if c.starts_with(self.input.as_str()) {
                let _ = self.filtered.push(c); // Ignore overflow
            }
        }
        self.tab_index = 0;
        if self.filtered.len() == 1 {
            self.input.clear();
            let _ = self.input.push_str(self.filtered[0]);
            let _ = self.input.push(' ');
        } else if self.filtered.len() > 1 {
            self.input = Self::longest_common_prefix(&self.filtered);
        }
    }

    /// cycles forward through filtered candidates.
    pub fn cycle_forward(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        self.tab_index = (self.tab_index + 1) % self.filtered.len();
        self.input.clear();
        let _ = self.input.push_str(self.filtered[self.tab_index]);
        let _ = self.input.push(' ');
    }

    /// cycles backward through filtered candidates.
    pub fn cycle_backward(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        self.tab_index = if self.tab_index == 0 {
            self.filtered.len() - 1
        } else {
            self.tab_index - 1
        };
        self.input.clear();
        let _ = self.input.push_str(self.filtered[self.tab_index]);
        let _ = self.input.push(' ');
    }

    /// Returns the current input string.
    pub fn current_input(&self) -> &str {
        &self.input
    }

    /// Finds the longest common prefix among the filtered candidates.
    fn longest_common_prefix(strings: &[&str]) -> String<FNL> {
        if strings.is_empty() {
            return String::new();
        }
        let mut prefix = strings[0];
        for s in strings.iter().skip(1) {
            while !s.starts_with(prefix) {
                if prefix.is_empty() {
                    break;
                }
                prefix = &prefix[..prefix.len() - 1];
            }
        }
        let mut result = String::new();
        let _ = result.push_str(prefix); // Ignore overflow
        result
    }

    /// Resets the input, filtered candidates, and tab index.
    pub fn reset(&mut self) {
        self.input.clear();
        self.filtered.clear();
        self.tab_index = 0;
    }
}
