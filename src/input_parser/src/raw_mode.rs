//#[cfg(unix)]

pub struct RawMode {
    #[cfg(unix)]
    original: termios::Termios,

    #[cfg(windows)]
    original_mode: u32,
}

impl RawMode {
    #[cfg(unix)]
    pub fn new(fd: i32) -> Self {
        use termios::*;
        let original = Termios::from_fd(fd).unwrap();
        let mut raw = original.clone();
        raw.c_lflag &= !(ICANON | ECHO);
        tcsetattr(fd, TCSANOW, &raw).unwrap();
        RawMode { original }
    }

    #[cfg(windows)]
    pub fn new(_: i32) -> Self {
        use winapi::um::consoleapi::*;
        use winapi::um::processenv::*;
        use winapi::um::wincon::*;
        use winapi::um::winnt::*;
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;

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

        unsafe {
            let handle = GetStdHandle(STD_INPUT_HANDLE);
            assert!(handle != INVALID_HANDLE_VALUE);
            SetConsoleMode(handle, self.original_mode);
        }
    }
}
