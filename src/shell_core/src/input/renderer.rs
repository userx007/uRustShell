use std::io::{self, Write};

/// DisplayRenderer: handles terminal output
///
pub struct DisplayRenderer;

impl DisplayRenderer {
    /// Renders the prompt and input content to the terminal.
    ///
    /// - Clears the current line.
    /// - Prints the prompt followed by the content.
    /// - Moves the cursor to the correct position based on `cursor_pos`.
    /// - Ensures cursor position does not exceed content length.
    /// - Flushes stdout to apply changes immediately.
    ///
    pub fn render(prompt: &str, content: &str, cursor_pos: usize) {
        let safe_cursor_pos = cursor_pos.min(content.len());
        print!("\r\x1B[K{}{}", prompt, content);
        print!("\x1B[{}G", prompt.len() + safe_cursor_pos + 1);
        let _ = io::stdout().flush();
    }

    /// Emits an audible bell sound in the terminal.
    ///
    /// - Useful for signaling invalid actions (e.g., backspace at start of buffer).
    /// - Flushes stdout to ensure the bell is triggered immediately.
    ///
    pub fn bell() {
        print!("\x07");
        let _ = io::stdout().flush();
    }

    /// Prints a red boundary marker in the terminal.
    ///
    /// - Displays a red newline character.
    /// - Moves the cursor back two positions.
    /// - Flushes stdout to apply changes immediately.
    /// - Can be used to visually separate sections or indicate limits.
    ///
    pub fn boundary_marker() {
        print!("\x1B[31m|\x1B[0m\x1B[1D \x1B[1D");
        let _ = io::stdout().flush();
    }
}

// ==================== TESTS =======================

#[cfg(test)]
mod tests {
    use super::*;

    // Test will just call methods to ensure they do not panic.
    // Because DisplayRenderer writes directly to stdout,
    // full capture is tricky without changing library code.
    #[test]
    fn test_render_does_not_panic() {
        DisplayRenderer::render(">", "Hello", 3);
    }

    #[test]
    fn test_bell_does_not_panic() {
        DisplayRenderer::bell();
    }

    #[test]
    fn test_boundary_marker_does_not_panic() {
        DisplayRenderer::boundary_marker();
    }
}
