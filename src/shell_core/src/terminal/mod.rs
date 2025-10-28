//! Cross-platform terminal raw mode handling.
//!
//! This module defines the `RawMode` struct, which enables and restores
//! raw mode for terminal input. Raw mode disables canonical input processing
//! and echo, allowing programs to read input byte-by-byte without waiting
//! for a newline and without echoing input to the terminal.
//!
//! The implementation is platform-specific:
//! - On **Unix**, it uses the `termios` crate to manipulate terminal attributes.
//! - On **Windows**, it uses the `winapi` crate to modify console modes.
//!
//! # Example
//! ```rust
//! // Enable raw mode
//! let _raw = RawMode::new(0); // 0 is the file descriptor for stdin
//! // Raw mode is active within this scope
//! // When `_raw` is dropped, the original mode is restored
//! ```

/// Represents a handle to the terminal's raw mode state.
/// When dropped, restores the original terminal mode.
pub struct RawMode {
    #[cfg(unix)]
    /// Original terminal settings (Unix).
    original: termios::Termios,
    #[cfg(windows)]
    /// Original console mode (Windows).
    original_mode: u32,
}

impl RawMode {
    /// Enables raw mode for the terminal.
    ///
    /// On Unix, `fd` is the file descriptor (usually 0 for stdin).
    /// On Windows, the argument is ignored.
    ///
    /// # Panics
    /// Panics if unable to get or set terminal/console mode.
    #[cfg(unix)]
    pub fn new(fd: i32) -> Self {
        use termios::*;
        let original = Termios::from_fd(fd).unwrap();
        let mut raw = original;
        raw.c_lflag &= !(ICANON | ECHO);
        tcsetattr(fd, TCSANOW, &raw).unwrap();
        RawMode { original }
    }

    #[cfg(windows)]
    pub fn new(_: i32) -> Self {
        use winapi::um::{
            consoleapi::{GetConsoleMode, SetConsoleMode},
            processenv::GetStdHandle,
            wincon::{ENABLE_LINE_INPUT, ENABLE_ECHO_INPUT},
            handleapi::INVALID_HANDLE_VALUE,
            winbase::STD_INPUT_HANDLE,
        };
        unsafe {
            let handle = GetStdHandle(STD_INPUT_HANDLE);
            assert!(handle != INVALID_HANDLE_VALUE);
            let mut mode = 0;
            GetConsoleMode(handle, &mut mode);
            let original_mode = mode;
            // Disable line input and echo
            mode &= !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);
            SetConsoleMode(handle, mode);
            RawMode { original_mode }
        }
    }
}

impl Drop for RawMode {
    /// Restores the original terminal/console mode when dropped.
    #[cfg(unix)]
    fn drop(&mut self) {
        use termios::*;
        tcsetattr(0, TCSANOW, &self.original).unwrap();
    }

    #[cfg(windows)]
    fn drop(&mut self) {
        use winapi::um::consoleapi::*;
        use winapi::um::processenv::*;
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;
        use winapi::um::winbase::STD_INPUT_HANDLE;
        unsafe {
            let handle = GetStdHandle(STD_INPUT_HANDLE);
            assert!(handle != INVALID_HANDLE_VALUE);
            SetConsoleMode(handle, self.original_mode);
        }
    }
}