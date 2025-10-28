use std::collections::VecDeque;

#[cfg(feature = "history-persistence")]
use std::fs::{OpenOptions, File};
#[cfg(feature = "history-persistence")]
use std::io::{BufRead, BufReader, Write};


pub const MAX_LEN: usize = 128;
pub const HISTORY_SIZE: usize = 50;

/*
#[cfg(feature = "history-persistence")]
struct Config {
    persist_history: bool,
    history_file: &'static str,
}
*/

//---------------------------------------------------------------------
pub struct History {
    buffer: VecDeque<String>,
    index: Option<usize>,
}

impl History {
    pub fn new() -> Self {
        History {
            buffer: VecDeque::with_capacity(HISTORY_SIZE),
            index: None,
        }
    }


    pub fn push(&mut self, entry: String) {
        if self.buffer.contains(&entry) {
            return; // Skip if already exists
        }

        if self.buffer.len() == HISTORY_SIZE {
            self.buffer.pop_front();
        }
        self.buffer.push_back(entry);
        self.index = None;
    }


    pub fn previous(&mut self) -> Option<&String> {
        match self.index {
            Some(i) if i > 0 => {
                self.index = Some(i - 1);
            }
            None if !self.buffer.is_empty() => {
                self.index = Some(self.buffer.len() - 1);
            }
            _ => {}
        }
        self.index.map(|i| &self.buffer[i])
    }


    pub fn next(&mut self) -> Option<&String> {
        match self.index {
            Some(i) if i + 1 < self.buffer.len() => {
                self.index = Some(i + 1);
                self.index.map(|i| &self.buffer[i])
            }
            _ => {
                self.index = None;
                None
            }
        }
    }


    pub fn list_with_indexes(&self) {
        if self.buffer.is_empty() {
            print!("⛔ history is empty");
        } else {
            print!("\n");
            self.buffer.iter().enumerate().for_each(|(index, value)| {print!("{:>3} : {}\n", index, value);});
        }
    }


    pub fn get_by_index(&self, idx: usize) -> Option<&String> {
        self.buffer.get(idx)
    }


    #[cfg(feature = "history-persistence")]
    pub fn load_from_file(&mut self, path: &str) {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten().take(HISTORY_SIZE) {
                self.push(line);
            }
        }
    }


    #[cfg(feature = "history-persistence")]
    pub fn append_to_file(&self, path: &str, entry: &str) {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{}", entry);
        }
    }
}
