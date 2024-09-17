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

use std::{cell::RefCell, collections::HashMap, num::NonZeroUsize, path::PathBuf, rc::Rc};

/*
TfIndexEntry -> TfEntry
IDFIndexEntry -> IdfEntry
*/

/*
file -> tokens -> token | inverted_index

TF:
file_index: -
frequency: -

IDF:
no of doc containing the term: -
total doc: -
*/

// use crate::util::Counter;

// During indexing, the Indexer will perform
// lookups in the hash table and create new IDFIndexEntries or
// TFIndexEntries if they don’t exist and update the frequency
// information for each term-file pair.
#[derive(Debug)]
pub struct Index {
    // TODO: FilePathIndex?
    // The FilePathIndex component is a memoryintensive component and is responsible for computing a file index from
    // the full file path and storing the index and the full file path into the
    // inverted index in order to be retrieved during search operations. The
    // file content, under the form of a list of extracted tokens, is then indexed by the TFIDFIndex, which is also a memory-intensive component
    // and that indexes the tokens and keeps track of the term frequencies
    // and inverse document frequencies necessary for computing the relevance score.
    pub file: FileIndex,

    pub inner: InvertedIndex,

    pub capacity: usize,

    pub threshold: usize,
}

impl Index {
    pub fn new(capacity: usize, threshold: usize) -> Self {
        Self {
            file: FileIndex::with_capacity(capacity),
            inner: InvertedIndex::with_capacity(capacity),
            capacity,
            threshold,
        }
    }

    pub fn insert(&mut self, term: String, path: String, word_count: usize) {
        // Add the file to the file index.
        let entry = FileEntry::new(path, word_count);
        let index = self.file.insert(entry);

        // insert into inverted index.

        // file index
        // number of times, the term occurs in the file.
        // let tf_entry = TfEntry::new(index);

        // capacity
        // threshold
        let idf_entry = IdfEntry::with_capacity(self.capacity, self.threshold);

        // self.inner.insert(term, )
    }
}

#[derive(Debug)]
pub struct FileIndex {
    inner: Vec<FileEntry>,
}

impl FileIndex {
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(1_000)
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn insert(&mut self, value: FileEntry) -> usize {
        self.inner.push(value);
        self.inner.len() - 1
    }
}

#[derive(Debug)]
pub struct FileEntry {
    path: String,

    // Word count
    count: NonZeroUsize,
}

impl FileEntry {
    pub fn new(path: String, word_count: usize) -> Self {
        Self {
            path,

            // SAFETY:
            // - The value must not be zero.
            // - Empty documents are not indexed.
            count: unsafe { NonZeroUsize::new_unchecked(word_count) },
        }
    }
}

#[derive(Debug)]
pub struct InvertedIndex {
    inner: HashMap<String, IdfEntry>,
}

impl InvertedIndex {
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(1_000)
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn insert(&mut self, term: String, file_index: usize, frequency: usize) {
        // self.inner.insert(term, entry);
        self.inner.entry(term).and_modify(|entry| {
            // IdfEntry is there

            let tf_entry = TfEntry::new(file_index, frequency);
            entry.insert(tf_entry);
        });
    }
}

// SCANNS
// IDFIndexEntry keeps track of the token associated with the entry,
// the number of files that contain the term,
#[derive(Debug, Default)]
pub struct IdfEntry {
    // Token
    // - maybe the key
    // token: String,

    // Number of files that contain the term.
    // - maybe the self.entries.len()
    //
    // file_count: Counter<usize>,
    entries: Vec<TfEntry>,

    // Limit after which the data will be flushed into the disk.
    threshold: usize,
}

impl IdfEntry {
    #[inline]
    pub fn with_capacity(capacity: usize, threshold: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            threshold,
        }
    }

    #[inline]
    /// Number of documents containing the term.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn insert(&mut self, entry: TfEntry) {
        self.entries.push(entry);
    }

    #[inline]
    pub fn should_flush(&self) -> bool {
        self.range() > self.threshold
    }

    #[inline]
    fn range(&self) -> usize {
        (self.entries.len() * 100) / self.entries.capacity()
    }
}

// SCANNS
// TFIndexEntry stores the index associated with
// a file, the frequency of a term in that file
#[derive(Debug, Default)]
pub struct TfEntry {
    /// Index associated with file.
    index: usize,

    /// Frequency of term in the file.
    frequency: usize,
}

impl TfEntry {
    pub fn new(index: usize, frequency: usize) -> Self {
        Self { index, frequency }
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
    use crate::index::{IdfEntry, TfEntry};

    use super::InvertedIndex;

    const INDEX_CAP: usize = 100;
    const CAPACITY: usize = 100;
    const THRESHOLD: usize = 80;

    #[test]
    fn test_idx_api() {
        let mut index = InvertedIndex::with_capacity(INDEX_CAP);

        let total_doc = 100;
        let document = "he is good boy".to_string();
        let term = "boy".to_string();

        // index.insert(term, )
    }

    #[test]
    fn test_index_idf_entry_basic() {
        let idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
        assert_eq!(idf.entries.capacity(), 100);
        // assert_eq!(*idf.file_count, 0);
        assert_eq!(idf.entries.len(), 0);
        assert_eq!(idf.threshold, 0);
    }

    // #[test]
    // fn test_index_idf_entry_with_file_count() {
    //     let idf = IdfEntry::new(100).with_file_count(Counter::new(5));
    //     assert_eq!(*idf.file_count, 5);
    // }

    #[test]
    fn test_index_idf_entry_with_limit() {
        let idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
        assert_eq!(idf.threshold, 80);
    }

    #[test]
    fn test_index_idf_entry_insert() {
        let mut idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
        idf.insert(TfEntry::new(1, 3));
        assert_eq!(idf.entries.len(), 1);
        assert_eq!(idf.entries[0].index, 1);
        assert_eq!(idf.entries[0].frequency, 3);
    }

    #[test]
    fn test_index_idf_entry_should_flush() {
        let mut idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
        (1..=81).for_each(|_| {
            idf.insert(TfEntry::default());
        });
        assert!(idf.should_flush());
    }

    #[test]
    fn test_index_idf_entry_should_not_flush() {
        let mut idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
        (1..=80).for_each(|_| {
            idf.insert(TfEntry::default());
        });
        assert!(!idf.should_flush());
    }

    #[test]
    fn test_index_idf_entry_range_calculation() {
        let mut idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
        (1..=50).for_each(|_| {
            idf.insert(TfEntry::default());
        });
        assert_eq!(idf.range(), 50);
    }
}
