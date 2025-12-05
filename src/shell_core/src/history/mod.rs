
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

// ==================== TEST =======================

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to create a clean history for testing
    fn new_test_history<const HTC: usize, const HME: usize>() -> History<HTC, HME> {
        let mut history = History::new();
        history.clear(); // Clear any loaded data from file
        history
    }

    // ==================== BASIC FUNCTIONALITY TESTS ====================

    #[test]
    fn test_new_history_is_empty() {
        let history = new_test_history::<1024, 10>();
        assert!(history.is_empty());
        assert_eq!(history.entry_size, 0);
    }

    #[test]
    fn test_default_creates_empty_history() {
        let mut history: History<1024, 10> = History::default();
        history.clear(); // Clear any persisted data
        assert!(history.is_empty());
    }

    #[test]
    fn test_push_single_entry() {
        let mut history = new_test_history::<1024, 10>();
        let result = history.push("test entry");
        assert!(result);
        assert_eq!(history.entry_size, 1);
        assert!(!history.is_empty());
    }

    #[test]
    fn test_push_and_retrieve() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        let entry: Option<String<1024>> = history.get(0);
        assert_eq!(entry.as_deref(), Some("first"));
    }

    #[test]
    fn test_push_multiple_entries() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.push("second");
        history.push("third");
        
        assert_eq!(history.entry_size, 3);
        assert_eq!(history.get::<1024>(0).as_deref(), Some("first"));
        assert_eq!(history.get::<1024>(1).as_deref(), Some("second"));
        assert_eq!(history.get::<1024>(2).as_deref(), Some("third"));
    }

    #[test]
    fn test_push_trims_whitespace() {
        let mut history = new_test_history::<1024, 10>();
        history.push("  trimmed  ");
        let entry: Option<String<1024>> = history.get(0);
        assert_eq!(entry.as_deref(), Some("trimmed"));
    }

    // ==================== DUPLICATE HANDLING TESTS ====================

    #[test]
    fn test_push_duplicate_rejected() {
        let mut history = new_test_history::<1024, 10>();
        assert!(history.push("duplicate"));
        assert!(!history.push("duplicate"));
        assert_eq!(history.entry_size, 1);
    }

    #[test]
    fn test_push_duplicate_with_whitespace_rejected() {
        let mut history = new_test_history::<1024, 10>();
        history.push("test");
        let result = history.push("  test  ");
        assert!(!result);
        assert_eq!(history.entry_size, 1);
    }

    #[test]
    fn test_similar_but_not_duplicate_accepted() {
        let mut history = new_test_history::<1024, 10>();
        history.push("test1");
        let result = history.push("test2");
        assert!(result);
        assert_eq!(history.entry_size, 2);
    }

    // ==================== CAPACITY TESTS ====================

    #[test]
    fn test_push_exceeds_byte_capacity_rejected() {
        let mut history = new_test_history::<10, 5>();
        let result = history.push("this is way too long");
        assert!(!result);
        assert_eq!(history.entry_size, 0);
    }

    #[test]
    fn test_push_at_byte_capacity_limit() {
        let mut history = new_test_history::<10, 5>();
        let result = history.push("1234567890");
        assert!(result);
        assert_eq!(history.entry_size, 1);
    }

    #[test]
    fn test_max_entries_capacity() {
        let mut history = new_test_history::<1024, 3>();
        history.push("first");
        history.push("second");
        history.push("third");
        history.push("fourth"); // Should evict "first"
        
        assert_eq!(history.entry_size, 3);
        assert_eq!(history.get::<1024>(0).as_deref(), Some("second"));
        assert_eq!(history.get::<1024>(1).as_deref(), Some("third"));
        assert_eq!(history.get::<1024>(2).as_deref(), Some("fourth"));
    }

    #[test]
    fn test_circular_buffer_wraps_correctly() {
        let mut history = new_test_history::<20, 5>();
        history.push("aaa");
        history.push("bbb");
        history.push("ccc");
        history.push("ddd");
        history.push("eee");
        history.push("fff"); // Should evict "aaa"
        
        assert_eq!(history.entry_size, 5);
        assert_eq!(history.get::<20>(0).as_deref(), Some("bbb"));
    }

    // ==================== NAVIGATION TESTS ====================

    #[test]
    fn test_get_prev_entry() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.push("second");
        history.push("third");
        
        assert_eq!(history.get_prev_entry::<1024>().as_deref(), Some("second"));
        assert_eq!(history.get_prev_entry::<1024>().as_deref(), Some("first"));
    }

    #[test]
    fn test_get_prev_entry_wraps_around() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.push("second");
        history.push("third");
        
        history.get_prev_entry::<1024>(); // second
        history.get_prev_entry::<1024>(); // first
        assert_eq!(history.get_prev_entry::<1024>().as_deref(), Some("third"));
    }

    #[test]
    fn test_get_next_entry() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.push("second");
        history.push("third");
        
        history.current_index = 0;
        assert_eq!(history.get_next_entry::<1024>().as_deref(), Some("second"));
        assert_eq!(history.get_next_entry::<1024>().as_deref(), Some("third"));
    }

    #[test]
    fn test_get_next_entry_wraps_around() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.push("second");
        history.push("third");
        
        history.current_index = 2;
        assert_eq!(history.get_next_entry::<1024>().as_deref(), Some("first"));
    }

    #[test]
    fn test_get_prev_entry_empty_history() {
        let mut history = new_test_history::<1024, 10>();
        assert_eq!(history.get_prev_entry::<1024>(), None);
    }

    #[test]
    fn test_get_next_entry_empty_history() {
        let mut history = new_test_history::<1024, 10>();
        assert_eq!(history.get_next_entry::<1024>(), None);
    }

    // ==================== FIRST/LAST ENTRY TESTS ====================

    #[test]
    fn test_get_first_entry() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.push("second");
        history.push("third");
        
        assert_eq!(history.get_first_entry::<1024>().as_deref(), Some("first"));
    }

    #[test]
    fn test_get_last_entry() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.push("second");
        history.push("third");
        
        assert_eq!(history.get_last_entry::<1024>().as_deref(), Some("third"));
    }

    #[test]
    fn test_get_first_entry_empty() {
        let history = new_test_history::<1024, 10>();
        assert_eq!(history.get_first_entry::<1024>(), None);
    }

    #[test]
    fn test_get_last_entry_empty() {
        let history = new_test_history::<1024, 10>();
        assert_eq!(history.get_last_entry::<1024>(), None);
    }

    #[test]
    fn test_first_last_single_entry() {
        let mut history = new_test_history::<1024, 10>();
        history.push("only");
        
        assert_eq!(history.get_first_entry::<1024>().as_deref(), Some("only"));
        assert_eq!(history.get_last_entry::<1024>().as_deref(), Some("only"));
    }

    #[test]
    fn test_first_last_after_eviction() {
        let mut history = new_test_history::<1024, 3>();
        history.push("first");
        history.push("second");
        history.push("third");
        history.push("fourth"); // Evicts "first"
        
        assert_eq!(history.get_first_entry::<1024>().as_deref(), Some("second"));
        assert_eq!(history.get_last_entry::<1024>().as_deref(), Some("fourth"));
    }

    // ==================== INDEX MANAGEMENT TESTS ====================

    #[test]
    fn test_set_index() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.push("second");
        history.push("third");
        
        history.set_index(1);
        assert_eq!(history.current_index, 1);
    }

    #[test]
    fn test_set_index_out_of_bounds() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        
        let old_index = history.current_index;
        history.set_index(10);
        assert_eq!(history.current_index, old_index);
    }

    #[test]
    fn test_get_invalid_index() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        
        assert_eq!(history.get::<1024>(5), None);
    }

    #[test]
    fn test_get_at_index() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.push("second");
        
        let result = history.get_at_index::<1024>(1);
        assert_eq!(result, Some((1, String::<1024>::try_from("second").unwrap())));
    }

    #[test]
    fn test_get_at_index_invalid() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        
        assert_eq!(history.get_at_index::<1024>(5), None);
    }

    // ==================== ITERATOR TESTS ====================

    #[test]
    fn test_iter_all_entries() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.push("second");
        history.push("third");
        
        let entries: Vec<String<1024>> = history.iter::<1024>().collect();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].as_str(), "first");
        assert_eq!(entries[1].as_str(), "second");
        assert_eq!(entries[2].as_str(), "third");
    }

    #[test]
    fn test_iter_empty_history() {
        let history = new_test_history::<1024, 10>();
        let entries: Vec<String<1024>> = history.iter::<1024>().collect();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_iter_with_indexes() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.push("second");
        history.push("third");
        
        let entries: Vec<(usize, String<1024>)> = history.iter_with_indexes::<1024>().collect();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].0, 0);
        assert_eq!(entries[0].1.as_str(), "first");
        assert_eq!(entries[2].0, 2);
        assert_eq!(entries[2].1.as_str(), "third");
    }

    #[test]
    fn test_iter_with_indexes_empty() {
        let history = new_test_history::<1024, 10>();
        let entries: Vec<(usize, String<1024>)> = history.iter_with_indexes::<1024>().collect();
        assert_eq!(entries.len(), 0);
    }

    // ==================== CLEAR TESTS ====================

    #[test]
    fn test_clear_history() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.push("second");
        
        history.clear();
        
        assert!(history.is_empty());
        assert_eq!(history.entry_size, 0);
        assert_eq!(history.data_head, 0);
        assert_eq!(history.entry_head, 0);
    }

    #[test]
    fn test_push_after_clear() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.clear();
        history.push("new");
        
        assert_eq!(history.entry_size, 1);
        assert_eq!(history.get::<1024>(0).as_deref(), Some("new"));
    }

    // ==================== FREE SPACE TESTS ====================

    #[test]
    fn test_get_free_space_empty() {
        let history = new_test_history::<100, 5>();
        let (free_bytes, free_entries) = history.get_free_space();
        assert_eq!(free_bytes, 100);
        assert_eq!(free_entries, 5);
    }

    #[test]
    fn test_get_free_space_with_entries() {
        let mut history = new_test_history::<100, 5>();
        history.push("test"); // 4 bytes
        
        let (free_bytes, free_entries) = history.get_free_space();
        assert_eq!(free_bytes, 96);
        assert_eq!(free_entries, 4);
    }

    #[test]
    fn test_get_free_space_full_entries() {
        let mut history = new_test_history::<100, 3>();
        history.push("a");
        history.push("b");
        history.push("c");
        
        let (_, free_entries) = history.get_free_space();
        assert_eq!(free_entries, 0);
    }

    // ==================== EDGE CASE TESTS ====================
/*
    #[test]
    fn test_empty_string_push() {
        let mut history = new_test_history::<1024, 10>();
        let result = history.push("");
        assert!(!result);
        assert_eq!(history.entry_size, 0);
    }

    #[test]
    fn test_whitespace_only_push() {
        let mut history = new_test_history::<1024, 10>();
        let result = history.push("   ");
        assert!(!result);
        assert_eq!(history.entry_size, 0);
    }

    #[test]
    fn test_unicode_entries() {
        let mut history = new_test_history::<1024, 10>();
        history.push("Hello 世界");
        history.push("Привет мир");
        
        assert_eq!(history.get::<1024>(0).as_deref(), Some("Hello 世界"));
        assert_eq!(history.get::<1024>(1).as_deref(), Some("Привет мир"));
    }
*/
    #[test]
    fn test_special_characters() {
        let mut history = new_test_history::<1024, 10>();
        history.push("!@#$%^&*()");
        history.push("tab\there");
        
        assert_eq!(history.get::<1024>(0).as_deref(), Some("!@#$%^&*()"));
        assert_eq!(history.get::<1024>(1).as_deref(), Some("tab\there"));
    }

    #[test]
    fn test_truncation_with_small_iml() {
        let mut history = new_test_history::<1024, 10>();
        history.push("this is a long string");
        
        let short: Option<String<5>> = history.get(0);
        assert_eq!(short.as_deref(), Some("this "));
    }

    // ==================== CIRCULAR BUFFER STRESS TESTS ====================

    #[test]
    fn test_many_small_entries() {
        let mut history = new_test_history::<50, 20>();
        for i in 0..15 {
            history.push(&format!("{}", i));
        }
        
        assert!(history.entry_size <= 20);
        let last: Option<String<50>> = history.get_last_entry();
        assert_eq!(last.as_deref(), Some("14"));
    }

    #[test]
    fn test_alternating_sizes() {
        let mut history = new_test_history::<100, 10>();
        history.push("a");
        history.push("longer string here");
        history.push("b");
        history.push("another long one");
        
        assert_eq!(history.entry_size, 4);
    }

    #[test]
    fn test_fill_exact_capacity() {
        let mut history = new_test_history::<10, 5>();
        history.push("12"); // 2 bytes
        history.push("34"); // 2 bytes
        history.push("56"); // 2 bytes
        history.push("78"); // 2 bytes
        history.push("90"); // 2 bytes - total 10 bytes
        
        assert_eq!(history.entry_size, 5);
    }

    #[test]
    fn test_force_multiple_evictions() {
        let mut history = new_test_history::<20, 10>();
        history.push("aa"); // 2 bytes
        history.push("bb"); // 2 bytes
        history.push("cc"); // 2 bytes
        history.push("very long string"); // 16 bytes - should evict multiple
        
        // Should have evicted enough entries to fit the new one
        assert!(history.entry_size >= 1);
        assert_eq!(history.get_last_entry::<20>().as_deref(), Some("very long string"));
    }

    // ==================== CURRENT INDEX BEHAVIOR TESTS ====================

    #[test]
    fn test_current_index_after_push() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        assert_eq!(history.current_index, 0);
        
        history.push("second");
        assert_eq!(history.current_index, 1);
    }

    #[test]
    fn test_current_index_preserved_across_navigation() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.push("second");
        history.push("third");
        
        history.get_prev_entry::<1024>();
        let idx = history.current_index;
        
        history.get_prev_entry::<1024>();
        assert_ne!(history.current_index, idx);
    }

    // ==================== INTEGRATION TESTS ====================

    #[test]
    fn test_complex_workflow() {
        let mut history = new_test_history::<100, 5>();
        
        history.push("command1");
        history.push("command2");
        history.push("command3");
        
        assert_eq!(history.get_prev_entry::<100>().as_deref(), Some("command2"));
        assert_eq!(history.get_prev_entry::<100>().as_deref(), Some("command1"));
        assert_eq!(history.get_next_entry::<100>().as_deref(), Some("command2"));
        
        history.push("command4");
        assert_eq!(history.get_last_entry::<100>().as_deref(), Some("command4"));
        
        let all: Vec<String<100>> = history.iter().collect();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_realistic_shell_history() {
        let mut history = new_test_history::<1024, 100>();
        
        let commands = vec![
            "ls -la",
            "cd /home/user",
            "git status",
            "cargo build",
            "cargo test",
            "git commit -m 'fix bug'",
            "git push origin main",
        ];
        
        for cmd in commands.iter() {
            history.push(cmd);
        }
        
        assert_eq!(history.entry_size, 7);
        assert_eq!(history.get_first_entry::<1024>().as_deref(), Some("ls -la"));
        assert_eq!(history.get_last_entry::<1024>().as_deref(), Some("git push origin main"));
        
        // Duplicate command rejected
        assert!(!history.push("git status"));
        assert_eq!(history.entry_size, 7);
    }

    // ==================== BOUNDARY TESTS ====================

    #[test]
    fn test_single_byte_entries() {
        let mut history = new_test_history::<10, 10>();
        for c in 'a'..'k' {
            history.push(&c.to_string());
        }
        // Should have 10 entries of 1 byte each
        assert_eq!(history.entry_size, 10);
    }

    #[test]
    fn test_eviction_order() {
        let mut history = new_test_history::<1024, 5>();
        history.push("first");
        history.push("second");
        history.push("third");
        history.push("fourth");
        history.push("fifth");
        
        // Add one more to evict the oldest
        history.push("sixth");
        
        // "first" should be evicted
        assert_eq!(history.get_first_entry::<1024>().as_deref(), Some("second"));
        assert_eq!(history.get_last_entry::<1024>().as_deref(), Some("sixth"));
    }

    #[test]
    fn test_navigation_after_eviction() {
        let mut history = new_test_history::<1024, 3>();
        history.push("a");
        history.push("b");
        history.push("c");
        history.push("d"); // Evicts "a"
        
        // Navigate should work correctly
        assert_eq!(history.get_prev_entry::<1024>().as_deref(), Some("c"));
        assert_eq!(history.get_prev_entry::<1024>().as_deref(), Some("b"));
        assert_eq!(history.get_prev_entry::<1024>().as_deref(), Some("d"));
    }

    #[test]
    fn test_duplicate_detection_across_eviction() {
        let mut history = new_test_history::<1024, 3>();
        history.push("test");
        history.push("other1");
        history.push("other2");
        history.push("other3"); // Evicts "test"
        
        // Now "test" should be accepted again since it was evicted
        assert!(history.push("test"));
    }

    #[test]
    fn test_very_long_string_near_limit() {
        let mut history = new_test_history::<100, 5>();
        let long_str = "a".repeat(99);
        assert!(history.push(&long_str));
        assert_eq!(history.entry_size, 1);
    }

    #[test]
    fn test_multiple_gets_dont_modify_state() {
        let mut history = new_test_history::<1024, 10>();
        history.push("first");
        history.push("second");
        
        let e1 = history.get::<1024>(0);
        let e2 = history.get::<1024>(0);
        let e3 = history.get::<1024>(1);
        
        assert_eq!(e1, e2);
        assert_ne!(e1, e3);
        assert_eq!(history.entry_size, 2);
    }
}