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

extern crate hashbrown;

use std::{cell::RefCell, hash::Hash, num::NonZeroUsize};

use hashbrown::{hash_map::HashMap, hash_set::HashSet};

use crate::{
    map::TermCounter,
    token::{Token, Tokens},
};

/*
TF:
file_index: -
frequency: -

IDF:
no of doc containing the term: -
total doc: -
*/

// TODO
// - Fix 7 bytes wasted (padding), due to tokenizer.
// - Lockfree data structures, to index and retrieve in separate threads, independently.

/// # Indexer
///
/// Currently, a single-threaded data structure implementation that internally utilizes
/// multithreading and SIMD for data-level parallelism,
/// optimizing throughput and tail latency.
///
/// The current strategy utilizes a single-threaded, thread-local indexer
/// and perform a merge operation to generate a global index view.
#[derive(Debug)]
pub struct Indexer {
    // The FilePathIndex component is a memory intensive component and is responsible for computing a file index from
    // the full file path and storing the index and the full file path into the
    // inverted index in order to be retrieved during search operations. The
    // file content, under the form of a list of extracted tokens, is then indexed by the TFIDFIndex, which is also a memory-intensive component
    // and that indexes the tokens and keeps track of the term frequencies
    // and inverse document frequencies necessary for computing the relevance score.
    pub core: CoreIndexer,

    pub capacity: usize,

    pub threshold: usize,
}

impl Indexer {
    pub fn new(capacity: usize, threshold: usize) -> Self {
        // TODO: Ensure threshold is less than capacity.

        Self {
            core: CoreIndexer::with_capacity(capacity),
            capacity,
            threshold,
        }
    }

    pub fn insert(&mut self, path: String, word_count: usize, tokens: &mut Tokens) {
        let mut file_entry = FileEntryBuilder::new(&mut self.core);
        let mut term_entry = file_entry.insert(path, word_count);

        tokens.iter_mut().for_each(|token| {
            term_entry.insert_with(|| {
                let term = std::mem::take(token);
                unsafe { std::ptr::drop_in_place(token) };
                term
            });
        });
    }
}

#[derive(Debug)]
pub struct CoreIndexer {
    store: FileIndex,
    index: InvertedIndex,
    count: TermCounter,
}

impl CoreIndexer {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            store: FileIndex::with_capacity(capacity),
            index: InvertedIndex::with_capacity(capacity),
            count: TermCounter::new(),
        }
    }

    /// SAFETY: The caller ensures that the counter is not zero, at least one term is inserted.
    unsafe fn get_word_frequency_unchecked(&self, term: &str) -> usize {
        unsafe { self.count.get_unchecked(term) }
    }
}

pub struct FileEntryBuilder<'cx> {
    indexer: &'cx mut CoreIndexer,
}

impl<'cx> FileEntryBuilder<'cx> {
    fn new(indexer: &'cx mut CoreIndexer) -> Self {
        Self { indexer }
    }

    pub fn insert<S: Into<String>>(
        &'cx mut self,
        path: S,
        word_count: usize,
    ) -> TermEntryBuilder<'cx> {
        let entry = FileEntry::new(path.into(), word_count);
        let index = self.insert_inner(entry);
        TermEntryBuilder::new(self.indexer, index)
    }

    fn insert_inner(&mut self, entry: FileEntry) -> usize {
        self.indexer.store.insert(entry)
    }
}

pub struct TermEntryBuilder<'cx> {
    indexer: &'cx mut CoreIndexer,
    file_index: usize,
}

impl<'cx> TermEntryBuilder<'cx> {
    fn new(indexer: &'cx mut CoreIndexer, file_index: usize) -> Self {
        Self {
            indexer,
            file_index,
        }
    }

    pub fn insert<S: Into<String>>(&mut self, term: S) {
        let term: String = term.into();

        self.insert_counter(term.clone());
        let word_frequency = unsafe { self.indexer.get_word_frequency_unchecked(&term) };
        let entry = TfEntry::new(self.file_index, word_frequency);

        self.insert_inner(term, entry);
    }

    pub fn insert_with(&mut self, f: impl FnOnce() -> Token) {
        let term = f();
        self.insert(term);
    }

    fn insert_inner(&mut self, term: String, entry: TfEntry) {
        self.indexer.index.insert(term, entry);
    }

    fn insert_counter(&mut self, term: String) {
        self.indexer.count.insert(term);
    }

    #[allow(unused)]
    fn reset_term_counter(&mut self) {
        self.indexer.count.reset();
    }
}

impl Drop for TermEntryBuilder<'_> {
    fn drop(&mut self) {
        self.indexer.count.reset();
    }
}

// TODO: Ensure that same files are not added more than once, maybe use another data structure.
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
    pub fn new<S: Into<String>>(path: S, word_count: usize) -> Self {
        Self {
            path: path.into(),

            // SAFETY:
            // - The value must not be zero.
            // - Empty documents are not indexed.
            count: unsafe { NonZeroUsize::new_unchecked(word_count) },
        }
    }
}

// TODO: Handle threshold.
#[derive(Debug)]
pub struct InvertedIndex {
    inner: HashMap<String, IdfEntry>,
}

impl InvertedIndex {
    #[inline]
    pub fn new(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn insert(&mut self, term: String, tf_entry: TfEntry) {
        // TODO: Track default capacity and threshold.
        self.inner
            .entry_ref(&term)
            .and_modify(|entry| entry.insert(RefEntry::new(tf_entry)))
            .or_insert_with(|| {
                let mut set = HashSet::new();
                set.insert(RefEntry::new(tf_entry));
                IdfEntry { entries: set }
            });

        // self.inner
        //     .raw_entry_mut()
        //     .from_key(&term)
        //     .and_modify(|_, entry| {
        //         entry.insert(RefEntry::new(tf_entry));
        //     })
        //     .or_insert_with(|| {
        //         let mut set = HashSet::new();
        //         set.insert(RefEntry::new(tf_entry));
        //         (term, IdfEntry { entries: set })
        //     })
        //     .0
        //     .as_str()
    }
}

// SCANNS
// IDFIndexEntry keeps track of the token associated with the entry,
// the number of files that contain the term,
#[derive(Debug, Default)]
pub struct IdfEntry {
    // TODO:
    // - Maybe use better data structure for this use-case.
    // - Track `threshold` for `IdfEntry` somewhere else.
    entries: HashSet<RefEntry>,
}

impl IdfEntry {
    #[inline]
    pub fn with_capacity(capacity: usize, threshold: usize) -> Self {
        Self {
            entries: HashSet::with_capacity(capacity),
        }
    }

    #[inline]
    /// Number of documents containing the term.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn insert(&mut self, entry: RefEntry) {
        if let Some(entry) = self.entries.get(&entry) {
            entry.0.borrow_mut().frequency += 1;
        }

        self.entries.insert(entry);
    }

    // #[inline]
    // pub fn should_flush(&self) -> bool {
    //     self.range() > self.threshold
    // }

    // #[inline]
    // fn range(&self) -> usize {
    //     (self.entries.len() * 100) / self.entries.capacity()
}

#[derive(Clone, Debug, Eq)]
pub struct RefEntry(RefCell<TfEntry>);

impl RefEntry {
    pub fn new(entry: TfEntry) -> Self {
        Self(RefCell::new(entry))
    }
}

impl Hash for RefEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.borrow().hash(state);
    }
}

impl PartialEq for RefEntry {
    fn eq(&self, other: &Self) -> bool {
        self.0.borrow().index == other.0.borrow().index
    }
}

// SCANNS
// TFIndexEntry stores the index associated with
// a file, the frequency of a term in that file
#[derive(Clone, Copy, Debug, Eq)]
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

impl Hash for TfEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}

impl PartialEq for TfEntry {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
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

    use super::{FileEntry, FileIndex, InvertedIndex};

    const INDEX_CAP: usize = 100;
    const CAPACITY: usize = 100;
    const THRESHOLD: usize = 80;

    #[test]
    fn test_idx_api() {
        // TODO:
        // 1. Document and File Path
        // 2. Insert in file index, get file_path_index
        // 3. Tokenize
        // 4. Normalize Pipeline
        // 5. Descriptor (document and file path)

        // for tokens/terms in document, insert to inverted index
        // 6. Insert in inverted index

        let mut index = InvertedIndex::with_capacity(INDEX_CAP);
        let mut file_index = FileIndex::with_capacity(INDEX_CAP);

        // let file_path_index = file_index.insert(FileEntry::new(path, word_count));
        // let mut tf_entry = TfEntry::new(file_path_index, frequency);

        // index.insert(term, tf_entry);

        let total_doc = 100;
        let document = "he is good boy".to_string();
        let term = "boy".to_string();

        // index.insert(term, )
    }

    // #[test]
    // fn test_index_idf_entry_basic() {
    //     let idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
    //     assert_eq!(idf.entries.capacity(), 100);
    //     // assert_eq!(*idf.file_count, 0);
    //     assert_eq!(idf.entries.len(), 0);
    //     assert_eq!(idf.threshold, 0);
    // }

    // // #[test]
    // // fn test_index_idf_entry_with_file_count() {
    // //     let idf = IdfEntry::new(100).with_file_count(Counter::new(5));
    // //     assert_eq!(*idf.file_count, 5);
    // // }

    // #[test]
    // fn test_index_idf_entry_with_limit() {
    //     let idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
    //     assert_eq!(idf.threshold, 80);
    // }

    // #[test]
    // fn test_index_idf_entry_insert() {
    //     let mut idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
    //     idf.insert(TfEntry::new(1, 3));
    //     assert_eq!(idf.entries.len(), 1);
    //     assert_eq!(idf.entries[0].index, 1);
    //     assert_eq!(idf.entries[0].frequency, 3);
    // }

    // #[test]
    // fn test_index_idf_entry_should_flush() {
    //     let mut idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
    //     (1..=81).for_each(|_| {
    //         idf.insert(TfEntry::default());
    //     });
    //     assert!(idf.should_flush());
    // }

    // #[test]
    // fn test_index_idf_entry_should_not_flush() {
    //     let mut idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
    //     (1..=80).for_each(|_| {
    //         idf.insert(TfEntry::default());
    //     });
    //     assert!(!idf.should_flush());
    // }

    // #[test]
    // fn test_index_idf_entry_range_calculation() {
    //     let mut idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
    //     (1..=50).for_each(|_| {
    //         idf.insert(TfEntry::default());
    //     });
    //     assert_eq!(idf.range(), 50);
    // }
}
