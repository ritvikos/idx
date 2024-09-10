use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::util::Counter;

#[derive(Debug)]
pub struct Index {
    pub inverted_index: InvertedIndex,
    pub file_index: FileIndex,
}

#[derive(Debug)]
pub struct FileIndex {}

#[derive(Debug)]
pub struct InvertedIndex {
    // Key = Token Index
    // Value = Index Entry
    inner: HashMap<usize, Rc<RefCell<IdfIndexEntry>>>,
}

#[derive(Debug, Default)]
pub struct IdfIndexEntry {
    /// Token
    token: String,

    /// Number of files that contain the term.
    file_count: Counter<usize>,

    entries: Vec<TfIndexEntry>,

    // Limit after which the data will be flushed into the disk.
    threshold: usize,
}

impl IdfIndexEntry {
    pub fn new(size: usize) -> Self {
        Self {
            entries: Vec::with_capacity(size),
            ..Default::default()
        }
    }

    pub fn with_file_count(mut self, count: Counter<usize>) -> Self {
        self.file_count = count;
        self
    }

    pub fn with_threshold(mut self, threshold: usize) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn insert(&mut self, term: TfIndexEntry) {
        self.entries.push(term);
    }

    pub fn should_flush(&self) -> bool {
        self.range() > self.threshold
    }

    fn range(&self) -> usize {
        (self.entries.len() * 100) / self.entries.capacity()
    }
}

#[derive(Debug, Default)]
pub struct TfIndexEntry {
    /// Index associated with file.
    file_index: usize,

    /// Frequency of term in the file.
    frequency: usize,
}

impl TfIndexEntry {
    pub fn new(file_index: usize, frequency: usize) -> Self {
        Self {
            file_index,
            frequency,
        }
    }
}

#[derive(Debug)]
pub struct AppendCache {
    last_file_index: usize,
    file_token_count: usize,
}

impl AppendCache {
    pub fn new(last_file_index: usize, file_token_count: usize) -> Self {
        Self {
            last_file_index,
            file_token_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        index::{IdfIndexEntry, TfIndexEntry},
        util::Counter,
    };

    #[test]
    fn test_index_idf_entry_basic() {
        let idf = IdfIndexEntry::new(100);
        assert_eq!(idf.entries.capacity(), 100);
        assert_eq!(*idf.file_count, 0);
        assert_eq!(idf.entries.len(), 0);
        assert_eq!(idf.threshold, 0);
    }

    #[test]
    fn test_index_idf_entry_with_file_count() {
        let idf = IdfIndexEntry::new(100).with_file_count(Counter::new(5));
        assert_eq!(*idf.file_count, 5);
    }

    #[test]
    fn test_index_idf_entry_with_limit() {
        let idf = IdfIndexEntry::new(100).with_threshold(80);
        assert_eq!(idf.threshold, 80);
    }

    #[test]
    fn test_index_idf_entry_insert() {
        let mut idf = IdfIndexEntry::new(100);
        idf.insert(TfIndexEntry::new(1, 3));
        assert_eq!(idf.entries.len(), 1);
        assert_eq!(idf.entries[0].file_index, 1);
        assert_eq!(idf.entries[0].frequency, 3);
    }

    #[test]
    fn test_index_idf_entry_should_flush() {
        let mut idf = IdfIndexEntry::new(100).with_threshold(80);
        (1..=81).for_each(|_| {
            idf.insert(TfIndexEntry::default());
        });
        assert!(idf.should_flush());
    }

    #[test]
    fn test_index_idf_entry_should_not_flush() {
        let mut idf = IdfIndexEntry::new(100).with_threshold(80);
        (1..=80).for_each(|_| {
            idf.insert(TfIndexEntry::default());
        });
        assert!(!idf.should_flush());
    }

    #[test]
    fn test_index_idf_entry_range_calculation() {
        let mut idf = IdfIndexEntry::new(100);
        (1..=50).for_each(|_| {
            idf.insert(TfIndexEntry::default());
        });
        assert_eq!(idf.range(), 50);
    }
}
