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