use heapless::{String, Vec};

/// Autocomplete struct for managing and filtering command candidates.
/// - `'a`: Lifetime for string slices.
/// - `NC`: Maximum number of candidates, NUM_COMMANDS.
/// - `FNL`: Maximum function name length, MAX_FUNCTION_NAME_LEN.
///
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
    ///
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
    ///
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
    ///
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
    ///
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
    ///
    pub fn current_input(&self) -> &str {
        &self.input
    }

    /// Finds the longest common prefix among the filtered candidates.
    ///
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
    ///
    pub fn reset(&mut self) {
        self.input.clear();
        self.filtered.clear();
        self.tab_index = 0;
    }
}

// ==================== TESTS =======================

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::{String, Vec};

    const NC: usize = 8;
    const FNL: usize = 32;

    fn make_candidates() -> Vec<&'static str, NC> {
        let mut v: Vec<&'static str, NC> = Vec::new();
        v.push("alpha").unwrap();
        v.push("alpine").unwrap();
        v.push("beta").unwrap();
        v.push("gamma").unwrap();
        v.push("gamut").unwrap();
        v.push("gambit").unwrap();
        v.push("zeta").unwrap();
        v
    }

    //----------------------------
    // Basic construction
    //----------------------------

    #[test]
    fn test_new() {
        let ac: Autocomplete<NC, FNL> = Autocomplete::new(make_candidates());
        assert_eq!(ac.current_input(), "");
        assert_eq!(ac.filtered.len(), 0);
        assert_eq!(ac.tab_index, 0);
    }

    //----------------------------
    // Filtering behavior
    //----------------------------

    #[test]
    fn test_filter_multiple() {
        let mut ac = Autocomplete::<NC, FNL>::new(make_candidates());
        let mut s: String<FNL> = String::new();
        s.push_str("ga").unwrap();

        ac.update_input(s);

        assert_eq!(ac.filtered.len(), 3); // gamma, gamut, gambit
        assert!(ac.filtered.contains(&"gamma"));
        assert!(ac.filtered.contains(&"gamut"));
        assert!(ac.filtered.contains(&"gambit"));
    }

    #[test]
    fn test_filter_none() {
        let mut ac = Autocomplete::<NC, FNL>::new(make_candidates());

        let mut s = String::<FNL>::new();
        s.push_str("xyz").unwrap();
        ac.update_input(s);

        assert_eq!(ac.filtered.len(), 0);
        assert_eq!(ac.current_input(), "xyz");
    }

    #[test]
    fn test_full_match_auto_complete() {
        let mut ac = Autocomplete::<NC, FNL>::new(make_candidates());

        let mut s = String::<FNL>::new();
        s.push_str("alp").unwrap();
        ac.update_input(s);

        // "alpha" and "alpine" match → multiple, so LCP computed
        assert_eq!(ac.filtered.len(), 2);
        assert_eq!(ac.current_input(), "alp");
    }

    //----------------------------
    // Single-match auto-complete
    //----------------------------

    #[test]
    fn test_single_match_auto_complete() {
        let mut ac = Autocomplete::<NC, FNL>::new(make_candidates());
        let mut s = String::<FNL>::new();
        s.push_str("bet").unwrap();

        ac.update_input(s);

        assert_eq!(ac.filtered.len(), 1);
        assert_eq!(ac.current_input(), "beta ");
    }

    //----------------------------
    // Longest Common Prefix Edge Cases
    //----------------------------

    #[test]
    fn test_lcp_no_common_prefix() {
        let strings = ["alpha", "beta", "gamma"];
        let result = Autocomplete::<NC, FNL>::longest_common_prefix(&strings);
        assert_eq!(result, "");
    }

    #[test]
    fn test_lcp_entire_word_common() {
        let strings = ["test", "testing", "tester"];
        let result = Autocomplete::<NC, FNL>::longest_common_prefix(&strings);
        assert_eq!(result, "test");
    }

    #[test]
    fn test_lcp_one_string() {
        let strings = ["hello"];
        let result = Autocomplete::<NC, FNL>::longest_common_prefix(&strings);
        assert_eq!(result, "hello");
    }

    //----------------------------
    // Cycling behavior
    //----------------------------

    #[test]
    fn test_cycle_forward_wrap() {
        let mut ac = Autocomplete::<NC, FNL>::new(make_candidates());

        let mut s = String::<FNL>::new();
        s.push_str("ga").unwrap();
        ac.update_input(s);

        ac.cycle_forward(); // index 1
        ac.cycle_forward(); // index 2
        ac.cycle_forward(); // wrap → index 0

        assert_eq!(ac.current_input(), "gamma ");
    }

    #[test]
    fn test_cycle_backward_wrap() {
        let mut ac = Autocomplete::<NC, FNL>::new(make_candidates());

        let mut s = String::<FNL>::new();
        s.push_str("ga").unwrap();
        ac.update_input(s);

        ac.cycle_backward(); // wrap to last
        assert_eq!(ac.current_input(), "gambit ");
    }

    #[test]
    fn test_cycle_no_filtered_candidates() {
        let mut ac = Autocomplete::<NC, FNL>::new(make_candidates());
        ac.cycle_forward(); // should not panic
        ac.cycle_backward(); // should not panic
        assert_eq!(ac.current_input(), "");
    }

    //----------------------------
    // Empty candidate list
    //----------------------------

    #[test]
    fn test_empty_candidate_list() {
        let empty: Vec<&'static str, NC> = Vec::new();
        let mut ac = Autocomplete::<NC, FNL>::new(empty);

        let mut s = String::<FNL>::new();
        s.push_str("a").unwrap();

        ac.update_input(s);
        assert_eq!(ac.filtered.len(), 0);
        assert_eq!(ac.current_input(), "a");
    }

    //----------------------------
    // Reset behavior
    //----------------------------

    #[test]
    fn test_reset() {
        let mut ac = Autocomplete::<NC, FNL>::new(make_candidates());
        let mut s = String::<FNL>::new();

        s.push_str("alp").unwrap();
        ac.update_input(s);

        ac.reset();
        assert_eq!(ac.current_input(), "");
        assert_eq!(ac.filtered.len(), 0);
        assert_eq!(ac.tab_index, 0);
    }

    //----------------------------
    // Overflow behavior
    //----------------------------

    #[test]
    fn test_filtered_overflow_graceful() {
        // Construct many items with same prefix
        let mut v: Vec<&'static str, 4> = Vec::new();
        v.push("abc").unwrap();
        v.push("abcd").unwrap();
        v.push("abcde").unwrap();
        v.push("abcdef").unwrap();

        let mut ac = Autocomplete::<4, FNL>::new(v);

        let mut s = String::<FNL>::new();
        s.push_str("a").unwrap();

        ac.update_input(s);

        // vec capacity is 4 → no overflow occurs, all candidates fit
        assert_eq!(ac.filtered.len(), 4);
        assert_eq!(ac.current_input(), "abc"); // LCP
    }

    #[test]
    fn test_candidate_list_overflow_handling() {
        let mut v: Vec<&'static str, 2> = Vec::new();
        v.push("alpha").unwrap();
        v.push("beta").unwrap();

        // now attempt (should not panic)
        let overflow_attempt = v.push("gamma");
        assert!(overflow_attempt.is_err());
    }

    //----------------------------
    // Fuzz-like deterministic randomized test
    //----------------------------

    #[test]
    fn test_fuzz_random_sequences() {
        let mut ac = Autocomplete::<NC, FNL>::new(make_candidates());

        let test_inputs = [
            "a", "al", "alp", "alpi", "g", "ga", "gam", "gamb", "z", "ze", "zet", "zeta",
        ];

        for inp in test_inputs {
            let mut s = String::<FNL>::new();
            s.push_str(inp).unwrap();
            ac.update_input(s);

            // Strong invariants:
            // 1. filtered only contains candidates that start with input prefix
            let prefix = inp;
            for f in ac.filtered.iter() {
                assert!(f.starts_with(prefix));
            }

            // 2. tab_index always valid
            if ac.filtered.len() > 0 {
                assert!(ac.tab_index < ac.filtered.len());
            } else {
                assert_eq!(ac.tab_index, 0);
            }
        }
    }
}
