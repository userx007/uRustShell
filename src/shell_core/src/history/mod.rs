#[cfg(feature = "history-persistence")]
extern crate std;
#[cfg(feature = "history-persistence")]
const HISTORY_FILENAME: &str  = ".hist";
#[cfg(feature = "history-persistence")]
use std::fmt::Write;

use heapless::String;

/// Metadata for a single entry in the history buffer.
/// Stores the offset and length of the entry in the circular buffer.
#[derive(Copy, Clone)]
pub struct EntryMeta {
    offset: usize,
    length: usize,
}

/// A fixed-size, circular history buffer for storing strings.
/// - `HTC`: History Total Capacity (bytes in buffer)
/// - `HME`: History Max Entries (number of entries)
pub struct History<const HTC: usize, const HME: usize> {
    data: [u8; HTC],
    entries: [Option<EntryMeta>; HME],
    data_head: usize,
    entry_head: usize,
    entry_size: usize,
    current_index: usize,
}

/// Iterator over history entries, yielding only the string values.
pub struct HistoryIter<'a, const HTC: usize, const HME: usize, const IML: usize> {
    history: &'a History<HTC, HME>,
    index: usize,
}

/// Iterator over history entries, yielding (index, string) pairs.
pub struct HistoryWithIndexesIter<'a, const HTC: usize, const HME: usize, const IML: usize> {
    history: &'a History<HTC, HME>,
    index: usize,
}

impl<const HTC: usize, const HME: usize> Default for History<HTC, HME> {
    /// Returns a new, empty history buffer.
    fn default() -> Self {
        Self::new()
    }
}

impl<const HTC: usize, const HME: usize> History<HTC, HME> {
    /// Creates a new, empty history buffer.
    pub fn new() -> Self {
        const NONE: Option<EntryMeta> = None;
        let instance = Self {
            data: [0; HTC],
            entries: [NONE; HME],
            data_head: 0,
            entry_head: 0,
            entry_size: 0,
            current_index: 0,
        };
        #[cfg(feature = "history-persistence")]
        let instance = {
            let mut inst = instance;
            inst.load_from_file(HISTORY_FILENAME);
            inst
        };
        instance
    }

    /// Pushes a new string into the history.
    /// - Trims whitespace.
    /// - Rejects if entry is too large or a duplicate.
    /// - Removes oldest entries if needed to make space.
    /// Returns `true` if the entry was added, `false` otherwise.
    pub fn push(&mut self, s: &str) -> bool {
        let trimmed = s.trim();
        let bytes = trimmed.as_bytes();
        let len = bytes.len();
        if len > HTC {
            return false;
        }
        // Check for duplicates
        for i in 0..self.entry_size {
            if let Some(existing) = self.get::<HTC>(i) {
                if existing.trim() == trimmed {
                    return false;
                }
            }
        }
        // Calculate used space
        let mut used = 0;
        for i in 0..self.entry_size {
            let idx = (self.entry_head + HME - self.entry_size + i) % HME;
            if let Some(meta) = self.entries[idx] {
                used += meta.length;
            }
        }
        let mut free = HTC - used;
        // Remove oldest entries until enough space is available
        while free < len && self.entry_size > 0 {
            let oldest_idx = (self.entry_head + HME - self.entry_size) % HME;
            if let Some(meta) = self.entries[oldest_idx] {
                free += meta.length;
            }
            self.entries[oldest_idx] = None;
            self.entry_size -= 1;
        }
        // Write new entry
        let offset = self.data_head;
        for (i, &b) in bytes.iter().enumerate() {
            self.data[(offset + i) % HTC] = b;
        }
        self.data_head = (self.data_head + len) % HTC;
        self.entries[self.entry_head] = Some(EntryMeta { offset, length: len });
        self.entry_head = (self.entry_head + 1) % HME;
        if self.entry_size < HME {
            self.entry_size += 1;
        }
        self.current_index = self.entry_size - 1;
        #[cfg(feature = "history-persistence")]
        self.append_to_file(HISTORY_FILENAME, trimmed);
        true
    }

    /// Moves to the previous entry and returns it, if any.
    pub fn get_prev_entry<const IML: usize>(&mut self) -> Option<String<IML>> {
        if self.entry_size == 0 {
            return None;
        }
        if self.current_index == 0 {
            self.current_index = self.entry_size - 1;
        } else {
            self.current_index -= 1;
        }
        self.get::<IML>(self.current_index)
    }

    /// Moves to the next entry and returns it, if any.
    pub fn get_next_entry<const IML: usize>(&mut self) -> Option<String<IML>> {
        if self.entry_size == 0 {
            return None;
        }
        self.current_index = (self.current_index + 1) % self.entry_size;
        self.get::<IML>(self.current_index)
    }


    /// Returns the **first (oldest)** entry in history, if any.
    pub fn get_first_entry<const IML: usize>(&self) -> Option<String<IML>> {
        if self.entry_size == 0 {
            return None;
        }
        // The oldest entry is at: (entry_head + HME - entry_size) % HME
        let oldest_idx = (self.entry_head + HME - self.entry_size) % HME;
        let meta = self.entries[oldest_idx]?;

        let mut s = String::<IML>::new();
        for i in 0..meta.length.min(IML) {
            let b = self.data[(meta.offset + i) % HTC];
            s.push(b as char).ok()?;
        }
        Some(s)
    }

    /// Returns the **last (most recent)** entry in history, if any.
    pub fn get_last_entry<const IML: usize>(&self) -> Option<String<IML>> {
        if self.entry_size == 0 {
            return None;
        }
        // The newest entry is just before entry_head (circularly)
        let newest_idx = (self.entry_head + HME - 1) % HME;
        let meta = self.entries[newest_idx]?;

        let mut s = String::<IML>::new();
        for i in 0..meta.length.min(IML) {
            let b = self.data[(meta.offset + i) % HTC];
            s.push(b as char).ok()?;
        }
        Some(s)
    }

    /// Sets the current index to the given value, if valid.
    pub fn set_index(&mut self, index: usize) {
        if index < self.entry_size {
            self.current_index = index;
        }
    }

    /// Returns `true` if the history is empty.
    pub fn is_empty(&self) -> bool {
        self.entry_size == 0
    }

    /// Gets the entry at the given index, if any.
    pub fn get<const IML: usize>(&self, index: usize) -> Option<String<IML>> {
        if index >= self.entry_size {
            return None;
        }
        let idx = (self.entry_head + HME - self.entry_size + index) % HME;
        let meta = self.entries[idx]?;
        let mut s = String::<IML>::new();
        for i in 0..meta.length.min(IML) {
            let b = self.data[(meta.offset + i) % HTC];
            s.push(b as char).ok()?;
        }
        Some(s)
    }

    /// Gets the entry and its index as a tuple, if any.
    pub fn get_at_index<const IML: usize>(&self, index: usize) -> Option<(usize, String<IML>)> {
        self.get::<IML>(index).map(|entry| (index, entry))
    }

    /// Returns an iterator over all entries.
    pub fn iter<const IML: usize>(&self) -> HistoryIter<'_, HTC, HME, IML> {
        HistoryIter {
            history: self,
            index: 0,
        }
    }

    /// Returns an iterator over all entries with their indexes.
    pub fn iter_with_indexes<const IML: usize>(&self) -> HistoryWithIndexesIter<'_, HTC, HME, IML> {
        HistoryWithIndexesIter {
            history: self,
            index: 0,
        }
    }

    /// Prints all entries and free space info to stdout.
    pub fn show<const IML: usize>(&self) {
        if self.is_empty() {
            println!("⚠️ History is empty");
        } else {
            self.iter_with_indexes::<IML>().for_each(|(index, entry)| {
                println!("{:>3} : {}", index, entry);
            });
            let (free_bytes, free_entries) = self.get_free_space();
            println!("📈 Left entries/bytes: {}/{}", free_entries, free_bytes);
        }
    }

    /// Clears all entries from the history.
    pub fn clear(&mut self) {
        self.data_head = 0;
        self.entry_head = 0;
        self.entry_size = 0;
        for e in self.entries.iter_mut() {
            *e = None;
        }
    }

    /// Returns the number of free bytes and free entry slots.
    pub fn get_free_space(&self) -> (usize, usize) {
        // Calculate used bytes in the circular buffer
        let used_bytes = if self.data_head
            >= self.entries[(self.entry_head + HME - self.entry_size) % HME]
                .map(|meta| meta.offset)
                .unwrap_or(self.data_head)
        {
            self.data_head
                - self.entries[(self.entry_head + HME - self.entry_size) % HME]
                    .map(|meta| meta.offset)
                    .unwrap_or(self.data_head)
        } else {
            HTC - (self.entries[(self.entry_head + HME - self.entry_size) % HME]
                .map(|meta| meta.offset)
                .unwrap_or(self.data_head)
                - self.data_head)
        };
        let free_bytes = HTC - used_bytes;
        let free_entries = HME - self.entry_size;
        (free_bytes, free_entries)
    }

    /// Loads history entries from a file (if `history-persistence` feature is enabled).
    #[cfg(feature = "history-persistence")]
    pub fn load_from_file(&mut self, path: &str) {
        use std::fs::File;
        use std::io::{BufReader, BufRead};
        use heapless::Vec;
        use heapless::String as HString;
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            let mut lines: Vec<HString<256>, HME> = Vec::new();
            for line_result in reader.lines() {
                if let Ok(line) = line_result {
                    if lines.len() == HME {
                        lines.remove(0);
                    }
                    let mut hl_line = HString::new();
                    let _ = write!(hl_line, "{}", line);
                    let _ = lines.push(hl_line);
                }
            }
            self.clear();
            for line in lines {
                let _ = self.push(&line);
            }
        }
    }

    #[cfg(feature = "history-persistence")]
    pub fn append_to_file(&self, path: &str, entry: &str) {
        use std::fs::OpenOptions;
        use std::io::Write;
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{}", entry);
        }
    }
}

/// Implements the `Iterator` trait for `HistoryIter`.
/// 
/// # Type Parameters
/// - `'a`: Lifetime of the iterator.
/// - `HTC`: History table capacity.
/// - `HME`: History max entries.
/// - `IML`: Item max length.
impl<'a, const HTC: usize, const HME: usize, const IML: usize> Iterator for HistoryIter<'a, HTC, HME, IML> {
    /// The type of item returned by the iterator.
    type Item = String<IML>;

    /// Advances the iterator and returns the next value.
    ///
    /// Returns `None` when all entries have been iterated.
    fn next(&mut self) -> Option<Self::Item> {
        // Check if we've reached the end of the history entries.
        if self.index >= self.history.entry_size {
            return None;
        }

        // Retrieve the current entry at `self.index`.
        let result = self.history.get::<IML>(self.index);
        // Move to the next entry for the next call.
        self.index += 1;
        result
    }
}

/// Implements the `Iterator` trait for `HistoryWithIndexesIter`.
///
/// This iterator yields both the index and the entry value.
/// 
/// # Type Parameters
/// - `'a`: Lifetime of the iterator.
/// - `HTC`: History table capacity.
/// - `HME`: History max entries.
/// - `IML`: Item max length.
impl<'a, const HTC: usize, const HME: usize, const IML: usize> Iterator for HistoryWithIndexesIter<'a, HTC, HME, IML> {
    /// The type of item returned by the iterator: a tuple of index and entry.
    type Item = (usize, String<IML>);

    /// Advances the iterator and returns the next (index, value) pair.
    ///
    /// Returns `None` when all entries have been iterated.
    fn next(&mut self) -> Option<Self::Item> {
        // Check if we've reached the end of the history entries.
        if self.index >= self.history.entry_size {
            return None;
        }

        // Retrieve the current entry and its index.
        let result = self.history.get_at_index::<IML>(self.index);
        // Move to the next entry for the next call.
        self.index += 1;
        result
    }
}