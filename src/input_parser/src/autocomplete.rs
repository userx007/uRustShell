pub struct Autocomplete<'a> {
    candidates: Vec<&'a str>,
    filtered: Vec<&'a str>,
    input: String,
    tab_index: usize,
}

impl<'a> Autocomplete<'a> {
    pub fn new(candidates: Vec<&'a str>) -> Self {
        Self {
            candidates,
            filtered: Vec::new(),
            input: String::new(),
            tab_index: 0,
        }
    }

    pub fn update_input(&mut self, new_input: String) {
        self.input = new_input;
        self.filtered = self
            .candidates
            .iter()
            .copied()
            .filter(|c| c.starts_with(&self.input))
            .collect();

        self.tab_index = 0;

        if self.filtered.len() == 1 {
            self.input = self.filtered[0].to_owned();
            self.input.push(' ');
        } else if self.filtered.len() > 1 {
            self.input = Self::longest_common_prefix(&self.filtered);
        }
    }

    // Tab key handler: autocomplete
    pub fn handle_tab(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        self.tab_index = (self.tab_index + 1) % self.filtered.len();
        self.input = self.filtered[self.tab_index].to_owned();
        self.input.push(' ');
    }

    // Shift-Tab key handler: autocomplete in reverse
    pub fn handle_shift_tab(&mut self) {
        if self.filtered.is_empty() {
            return;
        }

        // Decrement tab_index with wrap-around
        if self.tab_index == 0 {
            self.tab_index = self.filtered.len() - 1;
        } else {
            self.tab_index -= 1;
        }

        self.input = self.filtered[self.tab_index].to_owned();
        self.input.push(' ');
    }
/*
    pub fn handle_enter(&self) -> String {
        self.input.clone()
    }
*/
    pub fn current_input(&self) -> &str {
        &self.input
    }

    fn longest_common_prefix(strings: &[&str]) -> String {
        if strings.is_empty() {
            return String::new();
        }

        let mut prefix = strings[0];
        for s in strings.iter().skip(1) {
            while !s.starts_with(prefix) {
                prefix = &prefix[..prefix.len() - 1];
                if prefix.is_empty() {
                    break;
                }
            }
        }
        prefix.to_string()
    }

    pub fn reset(&mut self) {
        self.input.clear();
        self.filtered.clear();
        self.tab_index = 0;
    }
}