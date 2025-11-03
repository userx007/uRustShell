// keys.rs — cross-platform key reader

#[derive(Debug)]
pub enum Key {

    // Arrow keys – navigate through history or move the cursor
    ArrowUp,      // Move to previous history entry or move cursor up
    ArrowDown,    // Move to next history entry or move cursor down
    ArrowLeft,    // Move cursor left
    ArrowRight,   // Move cursor right

    // Navigation keys
    Home,         // Move cursor to the start of the line
    End,          // Move cursor to the end of the line
    Insert,       // Reserved / not currently used
    Delete,       // Delete character at the cursor position
    PageUp,       // Move to the oldest history entry
    PageDown,     // Move to the newest history entry

    // Input / editing keys
    Enter,        // Submit input or insert newline
    Backspace,    // Delete character before the cursor
    Tab,          // Navigate autocomplete forward
    ShiftTab,     // Navigate autocomplete backward

    // Control sequences for line editing
    CtrlU,        // Delete from cursor to beginning of line
    CtrlK,        // Delete from cursor to end of line
    CtrlD,        // Delete the entire line

    // Printable character
    Char(char),   // Any regular character input
}


#[cfg(windows)]
pub mod platform {
    use super::Key;
    use std::io;
    use winapi::um::consoleapi::ReadConsoleInputW;
    use winapi::um::wincon::{INPUT_RECORD, KEY_EVENT};
    use winapi::um::wincontypes::KEY_EVENT_RECORD;
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_INPUT_HANDLE;
    use winapi::shared::minwindef::DWORD;

    const LEFT_CTRL_PRESSED: u32 = 0x0008;
    const RIGHT_CTRL_PRESSED: u32 = 0x0004;
    const SHIFT_PRESSED: u32 = 0x0010;

    pub fn read_key() -> io::Result<Key> {
        unsafe {
            let handle = GetStdHandle(STD_INPUT_HANDLE);
            if handle.is_null() {
                return Err(io::Error::new(io::ErrorKind::Other, "Invalid handle"));
            }

            let mut record: INPUT_RECORD = std::mem::zeroed();
            let mut read: DWORD = 0;

            loop {
                if ReadConsoleInputW(handle, &mut record, 1, &mut read) == 0 {
                    return Err(io::Error::last_os_error());
                }

                if record.EventType == KEY_EVENT {
                    let key_event: KEY_EVENT_RECORD = *record.Event.KeyEvent();
                    if key_event.bKeyDown == 0 {
                        continue; // skip key release
                    }

                    let vkey = key_event.wVirtualKeyCode;
                    let c = *key_event.uChar.UnicodeChar() as u32;
                    let ctrl = (key_event.dwControlKeyState & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED)) != 0;
                    let shift = (key_event.dwControlKeyState & SHIFT_PRESSED) != 0;

                    // Handle Ctrl+ combos explicitly
                    if ctrl {
                        match vkey {
                            0x55 => return Ok(Key::CtrlU), // 'U'
                            0x4B => return Ok(Key::CtrlK), // 'K'
                            0x44 => return Ok(Key::CtrlD), // 'D'
                            _ => {}
                        }
                    }

                    // Map special keys
                    match vkey {
                        0x21 => return Ok(Key::PageUp),
                        0x22 => return Ok(Key::PageDown),
                        0x23 => return Ok(Key::End),
                        0x24 => return Ok(Key::Home),
                        0x25 => return Ok(Key::ArrowLeft),
                        0x26 => return Ok(Key::ArrowUp),
                        0x27 => return Ok(Key::ArrowRight),
                        0x28 => return Ok(Key::ArrowDown),
                        0x2E => return Ok(Key::Delete),
                        0x08 => return Ok(Key::Backspace),
                        0x09 => return Ok(if shift { Key::ShiftTab } else { Key::Tab }),
                        0x0D => return Ok(Key::Enter),
                        _ => {}
                    }

                    // Printable char — ignore NULs
                    if c != 0 {
                        return Ok(Key::Char(std::char::from_u32(c).unwrap_or('\0')));
                    }
                }
            }
        }
    }
}


#[cfg(not(windows))]
pub mod platform {
    use super::Key;
    use std::io::{self, Read};

    pub fn read_key() -> io::Result<Key> {
        let stdin = io::stdin();
        let mut bytes = stdin.lock().bytes();

        while let Some(Ok(b)) = bytes.next() {
            match b {
                b'\x1B' => { // Escape sequence
                    if let Some(Ok(b2)) = bytes.next() {
                        if b2 == b'[' {
                            if let Some(Ok(b3)) = bytes.next() {
                                return Ok(match b3 {
                                    b'A' => Key::ArrowUp,
                                    b'B' => Key::ArrowDown,
                                    b'C' => Key::ArrowRight,
                                    b'D' => Key::ArrowLeft,
                                    b'H' => Key::Home,
                                    b'F' => Key::End,
                                    b'Z' => Key::ShiftTab,
                                    b'1' | b'2' | b'3' | b'5' | b'6' => {
                                        // Read next '~' to confirm
                                        let _ = bytes.next();
                                        match b3 {
                                            b'1' => Key::Home,
                                            b'2' => Key::Insert,
                                            b'3' => Key::Delete,
                                            b'5' => Key::PageUp,
                                            b'6' => Key::PageDown,
                                            _ => Key::Char('~'),
                                        }
                                    }
                                    _ => Key::Char(b3 as char),
                                });
                            }
                        }
                    }
                }

                // Control keys
                b'\x15' => return Ok(Key::CtrlU), // Ctrl+U
                b'\x0B' => return Ok(Key::CtrlK), // Ctrl+K
                b'\x04' => return Ok(Key::CtrlD), // Ctrl+D

                // Normal keys
                b'\r' | b'\n' => return Ok(Key::Enter),
                b'\t' => return Ok(Key::Tab),
                b'\x7F' | b'\x08' => return Ok(Key::Backspace),
                c => return Ok(Key::Char(c as char)),
            }
        }

        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "No input"))
    }
}

/*
pub fn key_test() -> io::Result<()> {
    use std::io;
    println!("Press keys (Ctrl+C to exit)...");

    loop {
        match platform::read_key()? {
            Key::ArrowUp => println!("Arrow Up"),
            Key::ArrowDown => println!("Arrow Down"),
            Key::ArrowLeft => println!("Arrow Left"),
            Key::ArrowRight => println!("Arrow Right"),
            Key::Home => println!("Home"),
            Key::End => println!("End"),
            Key::Insert => println!("Insert"),
            Key::Delete => println!("Delete"),
            Key::PageUp => println!("Page Up"),
            Key::PageDown => println!("Page Down"),
            Key::Enter => println!("Enter"),
            Key::Backspace => println!("Backspace"),
            Key::Tab => println!("Tab"),
            Key::ShiftTab => println!("Shift+Tab"),
            Key::CtrlU => println!("Ctrl+U"),
            Key::CtrlK => println!("Ctrl+K"),
            Key::CtrlD => println!("Ctrl+D"),
            Key::Char(c) => println!("Char: {:?}", c),
        }
    }
}
*/
