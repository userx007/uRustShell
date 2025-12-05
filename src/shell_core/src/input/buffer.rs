use heapless::String;

/// A fixed-size, heapless character buffer for managing user input and cursor movement.
///
/// `InputBuffer` is ideal for embedded or resource-constrained environments where dynamic memory allocation is not desired.
/// It supports insertion, deletion, cursor movement, and conversion to a `heapless::String`.
///
/// # Type Parameters
/// - `IML`: The maximum input length (buffer size).
pub struct InputBuffer<const IML: usize> {
    buffer: [char; IML],
    length: usize,
    cursor_pos: usize,
}

impl<const IML: usize> InputBuffer<IML> {
    /// Creates a new, empty `InputBuffer` with the cursor at position 0.
    ///
    /// # Example
    /// ```
    /// let buf: InputBuffer<8> = InputBuffer::new();
    /// ```
    pub fn new() -> Self {
        Self {
            buffer: ['\0'; IML],
            length: 0,
            cursor_pos: 0,
        }
    }

    /// Inserts a character at the current cursor position.
    ///
    /// Shifts subsequent characters to the right.
    /// Returns `true` if the character was inserted, or `false` if the buffer is full.
    ///
    /// # Example
    /// ```
    /// let mut buf: InputBuffer<8> = InputBuffer::new();
    /// assert!(buf.insert('a'));
    /// ```
    pub fn insert(&mut self, ch: char) -> bool {
        if self.length >= IML {
            return false;
        }
        for i in (self.cursor_pos..self.length).rev() {
            self.buffer[i + 1] = self.buffer[i];
        }
        self.buffer[self.cursor_pos] = ch;
        self.length += 1;
        self.cursor_pos += 1;
        true
    }


    /// Deletes the character before the cursor (backspace).
    ///
    /// Returns `true` if a character was deleted, or `false` if at the start of the buffer.
    ///
    /// # Example
    /// ```
    /// let mut buf: InputBuffer<8> = InputBuffer::new();
    /// buf.insert('a');
    /// assert!(buf.backspace());
    /// ```
    pub fn backspace(&mut self) -> bool {
        if self.cursor_pos == 0 {
            return false;
        }
        for i in self.cursor_pos..self.length {
            self.buffer[i - 1] = self.buffer[i];
        }
        self.length -= 1;
        self.cursor_pos -= 1;
        self.buffer[self.length] = '\0';
        true
    }

    /// Moves the cursor one position to the left, if possible.
    ///
    /// # Example
    /// ```
    /// let mut buf: InputBuffer<8> = InputBuffer::new();
    /// buf.insert('a');
    /// buf.move_left();
    /// ```
    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    /// Moves the cursor one position to the right, if possible.
    ///
    /// # Example
    /// ```
    /// let mut buf: InputBuffer<8> = InputBuffer::new();
    /// buf.insert('a');
    /// buf.move_right();
    /// ```
    pub fn move_right(&mut self) {
        if self.cursor_pos < self.length {
            self.cursor_pos += 1;
        }
    }

    /// Moves the cursor to the start (home) of the buffer.
    ///
    /// # Example
    /// ```
    /// let mut buf: InputBuffer<8> = InputBuffer::new();
    /// buf.move_home();
    /// ```
    pub fn move_home(&mut self) {
        self.cursor_pos = 0;
    }

    /// Moves the cursor to the end of the buffer.
    ///
    /// # Example
    /// ```
    /// let mut buf: InputBuffer<8> = InputBuffer::new();
    /// buf.move_end();
    /// ```
    pub fn move_end(&mut self) {
        self.cursor_pos = self.length;
    }

    /// Deletes the character at the cursor position.
    ///
    /// # Example
    /// ```
    /// let mut buf: InputBuffer<8> = InputBuffer::new();
    /// buf.insert('a');
    /// buf.move_home();
    /// buf.delete_at_cursor();
    /// ```
    pub fn delete_at_cursor(&mut self) {
        if self.cursor_pos < self.length {
            for i in self.cursor_pos..self.length - 1 {
                self.buffer[i] = self.buffer[i + 1];
            }
            self.buffer[self.length - 1] = '\0';
            self.length -= 1;
        }
    }

    /// Clears the buffer and resets the cursor.
    ///
    /// # Example
    /// ```
    /// let mut buf: InputBuffer<8> = InputBuffer::new();
    /// buf.insert('a');
    /// buf.clear();
    /// ```
    pub fn clear(&mut self) {
        self.buffer = ['\0'; IML];
        self.length = 0;
        self.cursor_pos = 0;
    }

    /// Returns the buffer contents as a `heapless::String`.
    ///
    /// # Example
    /// ```
    /// let mut buf: InputBuffer<8> = InputBuffer::new();
    /// buf.insert('a');
    /// let s = buf.to_string();
    /// ```
    pub fn to_string(&self) -> String<IML> {
        self.buffer.iter().take(self.length).collect()
    }

    /// Overwrites the buffer with the given string, truncating if necessary.
    ///
    /// The cursor is moved to the end of the new content.
    ///
    /// # Example
    /// ```
    /// let mut buf: InputBuffer<8> = InputBuffer::new();
    /// buf.overwrite("hello");
    /// ```
    pub fn overwrite(&mut self, input: &str) {
        self.clear();
        for (i, c) in input.chars().take(IML).enumerate() {
            self.buffer[i] = c;
        }
        self.length = input.len().min(IML);
        self.cursor_pos = self.length;
    }

    /// Returns the current cursor position.
    ///
    /// # Example
    /// ```
    /// let buf: InputBuffer<8> = InputBuffer::new();
    /// let pos = buf.cursor();
    /// ```
    pub fn cursor(&self) -> usize {
        self.cursor_pos
    }

    /// Deletes all characters from the start up to the cursor.
    ///
    /// The cursor is moved to the start.
    ///
    /// # Example
    /// ```
    /// let mut buf: InputBuffer<8> = InputBuffer::new();
    /// buf.overwrite("hello");
    /// buf.move_right();
    /// buf.delete_to_start();
    /// ```
    pub fn delete_to_start(&mut self) {
        let shift = self.length - self.cursor_pos;
        for i in 0..shift {
            self.buffer[i] = self.buffer[self.cursor_pos + i];
        }
        for i in shift..IML {
            self.buffer[i] = '\0';
        }
        self.length = shift;
        self.cursor_pos = 0;
    }

    /// Deletes all characters from the cursor to the end.
    ///
    /// # Example
    /// ```
    /// let mut buf: InputBuffer<8> = InputBuffer::new();
    /// buf.overwrite("hello");
    /// buf.move_home();
    /// buf.delete_to_end();
    /// ```
    pub fn delete_to_end(&mut self) {
        for i in self.cursor_pos..self.length {
            self.buffer[i] = '\0';
        }
        self.length = self.cursor_pos;
    }


    /// Returns the current length of the buffer.
    ///
    /// # Example
    /// ```
    /// let buf: InputBuffer<8> = InputBuffer::new();
    /// let len = buf.len();
    /// ```
    pub fn len(&self) -> usize {
        self.length
    }

    /// Returns `true` if the buffer is empty.
    ///
    /// # Example
    /// ```
    /// let buf: InputBuffer<8> = InputBuffer::new();
    /// assert!(buf.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

}

impl<const IML: usize> Default for InputBuffer<IML> {
    /// Returns a new, empty `InputBuffer`.
    fn default() -> Self {
        Self::new()
    }
}

// ==================== TEST =======================
#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Basic Creation and Initialization
    // ============================================================================

    #[test]
    fn test_new_creates_empty_buffer() {
        let buf: InputBuffer<8> = InputBuffer::new();
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.cursor(), 0);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_default_trait() {
        let buf: InputBuffer<16> = InputBuffer::default();
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.cursor(), 0);
    }

    // ============================================================================
    // Insert Operations
    // ============================================================================

    #[test]
    fn test_insert_single_char() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        assert!(buf.insert('a'));
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.cursor(), 1);
        assert_eq!(buf.to_string().as_str(), "a");
    }

    #[test]
    fn test_insert_multiple_chars() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        assert!(buf.insert('h'));
        assert!(buf.insert('e'));
        assert!(buf.insert('l'));
        assert!(buf.insert('l'));
        assert!(buf.insert('o'));
        assert_eq!(buf.to_string().as_str(), "hello");
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.cursor(), 5);
    }

    #[test]
    fn test_insert_at_buffer_capacity() {
        let mut buf: InputBuffer<4> = InputBuffer::new();
        assert!(buf.insert('a'));
        assert!(buf.insert('b'));
        assert!(buf.insert('c'));
        assert!(buf.insert('d'));
        assert_eq!(buf.len(), 4);
    }

    #[test]
    fn test_insert_beyond_capacity_fails() {
        let mut buf: InputBuffer<3> = InputBuffer::new();
        assert!(buf.insert('a'));
        assert!(buf.insert('b'));
        assert!(buf.insert('c'));
        assert!(!buf.insert('d')); // Should fail
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.to_string().as_str(), "abc");
    }

    #[test]
    fn test_insert_unicode_chars() {
        let mut buf: InputBuffer<10> = InputBuffer::new();
        assert!(buf.insert('α'));
        assert!(buf.insert('β'));
        assert!(buf.insert('γ'));
        assert!(buf.insert('🦀'));
        assert_eq!(buf.len(), 4);
    }

    #[test]
    fn test_insert_in_middle() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.insert('a');
        buf.insert('c');
        buf.move_left();
        assert!(buf.insert('b'));
        assert_eq!(buf.to_string().as_str(), "abc");
        assert_eq!(buf.cursor(), 2);
    }

    #[test]
    fn test_insert_at_start() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.insert('a');
        buf.insert('b');
        buf.move_home();
        assert!(buf.insert('x'));
        assert_eq!(buf.to_string().as_str(), "xab");
        assert_eq!(buf.cursor(), 1);
    }

    // ============================================================================
    // Backspace Operations
    // ============================================================================

    #[test]
    fn test_backspace_removes_char() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.insert('a');
        buf.insert('b');
        assert!(buf.backspace());
        assert_eq!(buf.to_string().as_str(), "a");
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.cursor(), 1);
    }

    #[test]
    fn test_backspace_at_start_fails() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.insert('a');
        buf.move_home();
        assert!(!buf.backspace());
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn test_backspace_empty_buffer_fails() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        assert!(!buf.backspace());
    }

    #[test]
    fn test_backspace_in_middle() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.move_home();
        buf.move_right();
        buf.move_right();
        assert!(buf.backspace());
        assert_eq!(buf.to_string().as_str(), "hllo");
        assert_eq!(buf.cursor(), 1);
    }

    #[test]
    fn test_multiple_backspaces() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("test");
        assert!(buf.backspace());
        assert!(buf.backspace());
        assert_eq!(buf.to_string().as_str(), "te");
        assert_eq!(buf.len(), 2);
    }

    // ============================================================================
    // Cursor Movement
    // ============================================================================

    #[test]
    fn test_move_left() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.insert('a');
        buf.insert('b');
        buf.move_left();
        assert_eq!(buf.cursor(), 1);
    }

    #[test]
    fn test_move_left_at_start() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.insert('a');
        buf.move_home();
        buf.move_left();
        assert_eq!(buf.cursor(), 0);
    }

    #[test]
    fn test_move_right() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.insert('a');
        buf.insert('b');
        buf.move_left();
        buf.move_right();
        assert_eq!(buf.cursor(), 2);
    }

    #[test]
    fn test_move_right_at_end() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.insert('a');
        buf.move_right();
        assert_eq!(buf.cursor(), 1);
    }

    #[test]
    fn test_move_home() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.move_home();
        assert_eq!(buf.cursor(), 0);
    }

    #[test]
    fn test_move_home_on_empty() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.move_home();
        assert_eq!(buf.cursor(), 0);
    }

    #[test]
    fn test_move_end() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.move_home();
        buf.move_end();
        assert_eq!(buf.cursor(), 5);
    }

    #[test]
    fn test_move_end_on_empty() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.move_end();
        assert_eq!(buf.cursor(), 0);
    }

    // ============================================================================
    // Delete at Cursor
    // ============================================================================

    #[test]
    fn test_delete_at_cursor() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.move_home();
        buf.delete_at_cursor();
        assert_eq!(buf.to_string().as_str(), "ello");
        assert_eq!(buf.cursor(), 0);
    }

    #[test]
    fn test_delete_at_cursor_middle() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.move_home();
        buf.move_right();
        buf.move_right();
        buf.delete_at_cursor();
        assert_eq!(buf.to_string().as_str(), "helo");
        assert_eq!(buf.cursor(), 2);
    }

    #[test]
    fn test_delete_at_cursor_end() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.delete_at_cursor();
        assert_eq!(buf.to_string().as_str(), "hello");
    }

    #[test]
    fn test_delete_at_cursor_empty() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.delete_at_cursor();
        assert_eq!(buf.len(), 0);
    }

    // ============================================================================
    // Clear Operation
    // ============================================================================

    #[test]
    fn test_clear() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.clear();
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.cursor(), 0);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_clear_empty_buffer() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.clear();
        assert_eq!(buf.len(), 0);
    }

    // ============================================================================
    // Overwrite Operation
    // ============================================================================

    #[test]
    fn test_overwrite() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        assert_eq!(buf.to_string().as_str(), "hello");
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.cursor(), 5);
    }

    #[test]
    fn test_overwrite_replaces_content() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.overwrite("hi");
        assert_eq!(buf.to_string().as_str(), "hi");
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_overwrite_truncates() {
        let mut buf: InputBuffer<4> = InputBuffer::new();
        buf.overwrite("hello");
        assert_eq!(buf.to_string().as_str(), "hell");
        assert_eq!(buf.len(), 4);
    }

    #[test]
    fn test_overwrite_empty_string() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.overwrite("");
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
    }


    // ============================================================================
    // Delete to Start
    // ============================================================================

    #[test]
    fn test_delete_to_start() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.move_home();
        buf.move_right();
        buf.move_right();
        buf.delete_to_start();
        assert_eq!(buf.to_string().as_str(), "llo");
        assert_eq!(buf.cursor(), 0);
    }

    #[test]
    fn test_delete_to_start_at_start() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.move_home();
        buf.delete_to_start();
        assert_eq!(buf.to_string().as_str(), "hello");
    }

    #[test]
    fn test_delete_to_start_at_end() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.delete_to_start();
        assert_eq!(buf.to_string().as_str(), "");
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_delete_to_start_empty() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.delete_to_start();
        assert_eq!(buf.len(), 0);
    }

    // ============================================================================
    // Delete to End
    // ============================================================================

    #[test]
    fn test_delete_to_end() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.move_home();
        buf.move_right();
        buf.move_right();
        buf.delete_to_end();
        assert_eq!(buf.to_string().as_str(), "he");
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_delete_to_end_at_end() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.delete_to_end();
        assert_eq!(buf.to_string().as_str(), "hello");
    }

    #[test]
    fn test_delete_to_end_at_start() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.move_home();
        buf.delete_to_end();
        assert_eq!(buf.to_string().as_str(), "");
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_delete_to_end_empty() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.delete_to_end();
        assert_eq!(buf.len(), 0);
    }

    // ============================================================================
    // Complex Scenarios
    // ============================================================================

    #[test]
    fn test_complex_editing_scenario() {
        let mut buf: InputBuffer<16> = InputBuffer::new();
        buf.overwrite("hello world");
        buf.move_home();
        buf.move_right();
        buf.move_right();
        buf.move_right();
        buf.move_right();
        buf.move_right();
        buf.insert('_');
        buf.move_end();
        buf.backspace();
        buf.backspace();
        buf.backspace();
        buf.backspace();
        buf.backspace();
        buf.backspace();
        assert_eq!(buf.to_string().as_str(), "hello_");
    }

    #[test]
    fn test_insert_after_backspace() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.insert('a');
        buf.insert('b');
        buf.insert('c');
        buf.backspace();
        buf.insert('x');
        assert_eq!(buf.to_string().as_str(), "abx");
    }

    #[test]
    fn test_cursor_navigation_and_edit() {
        let mut buf: InputBuffer<10> = InputBuffer::new();
        buf.overwrite("test");
        buf.move_home();
        buf.move_right();
        buf.move_right();
        buf.delete_at_cursor();
        buf.insert('x');
        assert_eq!(buf.to_string().as_str(), "text");
    }

    #[test]
    fn test_full_buffer_operations() {
        let mut buf: InputBuffer<4> = InputBuffer::new();
        assert!(buf.insert('a'));
        assert!(buf.insert('b'));
        assert!(buf.insert('c'));
        assert!(buf.insert('d'));
        assert!(!buf.insert('e'));
        buf.backspace();
        assert!(buf.insert('x'));
        assert_eq!(buf.to_string().as_str(), "abcx");
    }

    #[test]
    fn test_alternating_insert_and_delete() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.insert('a');
        buf.delete_at_cursor();
        buf.insert('b');
        buf.move_left();
        buf.delete_at_cursor();
        buf.insert('c');
        assert_eq!(buf.to_string().as_str(), "ac");
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_zero_capacity_buffer() {
        let mut buf: InputBuffer<0> = InputBuffer::new();
        assert!(!buf.insert('a'));
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_single_capacity_buffer() {
        let mut buf: InputBuffer<1> = InputBuffer::new();
        assert!(buf.insert('a'));
        assert!(!buf.insert('b'));
        assert_eq!(buf.to_string().as_str(), "a");
    }

    #[test]
    fn test_whitespace_characters() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.insert(' ');
        buf.insert('\t');
        buf.insert('\n');
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn test_null_character() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.insert('\0');
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn test_repeated_operations() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        for _ in 0..100 {
            buf.move_home();
            buf.move_end();
        }
        assert_eq!(buf.cursor(), 0);
    }

    // ============================================================================
    // State Consistency Tests
    // ============================================================================

    #[test]
    fn test_cursor_never_exceeds_length() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("test");
        buf.move_end();
        buf.move_right();
        buf.move_right();
        assert!(buf.cursor() <= buf.len());
    }

    #[test]
    fn test_length_consistency_after_operations() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.insert('a');
        buf.insert('b');
        buf.insert('c');
        let len = buf.len();
        assert_eq!(buf.to_string().len(), len);
    }

    #[test]
    fn test_buffer_state_after_clear() {
        let mut buf: InputBuffer<8> = InputBuffer::new();
        buf.overwrite("hello");
        buf.move_home();
        buf.move_right();
        buf.clear();
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.cursor(), 0);
        assert_eq!(buf.to_string().as_str(), "");
    }
}