// Persistent Index

// file path index stored under: file_index_<thread_id> dir.
// tfidf index stored under:     tfidf_index_<thread_id> dir.

// For each flush of any index type,
// one segment data file and one of more segment metadata files are created.

// Segment metadata files contain hashtable entries for each index type.
// Example: TFIDF Index segment metadata will contain list of IFDIndexEntry elements (array) in binary format.
//
// Segment data files contain auxiliary data structure elements.
// In FilePathIndex, the auxiliary structure is actual full file path and corresponding element
// from metadata file stores an offset to position where full file path can be found in the
// data file.
// TFIDFIndex, segment data file contains a list of lists of TFIndexEntry elements of size IndexDepth

use std::{cell::RefCell, collections::HashMap, rc::Rc};

// use crate::util::Counter;

// During indexing, the Indexer will perform
// lookups in the hash table and create new IDFIndexEntries or
// TFIndexEntries if they don’t exist and update the frequency
// information for each term-file pair.
#[derive(Debug)]
pub struct Index {
    pub inverted_index: InvertedIndex,

    // TODO: FilePathIndex?
    // The FilePathIndex component is a memoryintensive component and is responsible for computing a file index from
    // the full file path and storing the index and the full file path into the
    // inverted index in order to be retrieved during search operations. The
    // file content, under the form of a list of extracted tokens, is then indexed by the TFIDFIndex, which is also a memory-intensive component
    // and that indexes the tokens and keeps track of the term frequencies
    // and inverse document frequencies necessary for computing the relevance score.
    pub file_index: FileIndex,
}

#[derive(Debug)]
pub struct FileIndex {}

#[derive(Debug)]
pub struct InvertedIndex {
    // Key = Token Index
    // Value = Index Entry
    inner: HashMap<String, Rc<RefCell<IdfIndexEntry>>>,
}

// SCANNS
// IDFIndexEntry keeps track of the token associated with the entry,
// the number of files that contain the term,
#[derive(Debug, Default)]
pub struct IdfIndexEntry {
    // Token
    // - maybe the key
    // token: String,

    // Number of files that contain the term.
    // - maybe the self.entries.len()
    //
    // file_count: Counter<usize>,
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

    // pub fn with_file_count(mut self, count: Counter<usize>) -> Self {
    //     self.file_count = count;
    //     self
    // }

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

// SCANNS
// TFIndexEntry stores the index associated with
// a file, the frequency of a term in that file
#[derive(Debug, Default)]
pub struct TfIndexEntry {
    /// Index associated with file.
    file_index: usize,

    /// Frequency of term in the file.
    frequency: usize,
    // ---
    // Not mentioned in paper.
    // total file tokens param
    // can be added here only
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
        // util::Counter,
    };

    #[test]
    fn test_index_idf_entry_basic() {
        let idf = IdfIndexEntry::new(100);
        assert_eq!(idf.entries.capacity(), 100);
        // assert_eq!(*idf.file_count, 0);
        assert_eq!(idf.entries.len(), 0);
        assert_eq!(idf.threshold, 0);
    }

    // #[test]
    // fn test_index_idf_entry_with_file_count() {
    //     let idf = IdfIndexEntry::new(100).with_file_count(Counter::new(5));
    //     assert_eq!(*idf.file_count, 5);
    // }

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
