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

use std::{borrow::Borrow, cell::RefCell, hash::Hash, marker::PhantomData, num::NonZeroUsize};

use hashbrown::{hash_map::HashMap, hash_set::HashSet};

use crate::{
    map::TermCounter,
    token::{Token, Tokens},
    util::Counter,
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
pub struct Index {
    // The FilePathIndex component is a memory intensive component and is responsible for computing a file index from
    // the full file path and storing the index and the full file path into the
    // inverted index in order to be retrieved during search operations. The
    // file content, under the form of a list of extracted tokens, is then indexed by the TFIDFIndex, which is also a memory-intensive component
    // and that indexes the tokens and keeps track of the term frequencies
    // and inverse document frequencies necessary for computing the relevance score.
    pub core: CoreIndex,

    pub capacity: usize,

    pub threshold: usize,
}

impl Index {
    /// Creates a new instance of `Index`
    pub fn new(capacity: usize, threshold: usize) -> Self {
        // TODO: Ensure threshold is less than capacity.

        Self {
            core: CoreIndex::with_capacity(capacity),
            capacity,
            threshold,
        }
    }

    pub fn insert(&mut self, path: String, word_count: usize, tokens: &mut Tokens) {
        let writer = self.core.writer();
        let file_entry = WriterContext::<FileEntryState>::new(writer);

        let mut term_entry = file_entry.entry(path, word_count);

        tokens.iter_mut().for_each(|token| {
            term_entry.insert_term_with(|| {
                let term = std::mem::take(token);
                unsafe { std::ptr::drop_in_place(token) };
                term
            });
        });

        term_entry.reset_counter()
    }

    /// Number of documents containing the term.
    #[inline]
    pub fn doc_term_frequency(&self, term: &str) -> Option<usize> {
        match self.get_term_entries(term) {
            Some(entry) => Some(entry.count()),
            None => None,
        }
    }

    /// Number of indexed documents.
    #[inline]
    pub fn total_docs(&self) -> usize {
        self.core.total_docs()
    }

    /// Number of indexed entries for a term.
    #[inline]
    fn get_term_entries(&self, term: &str) -> Option<&IdfEntry> {
        self.core.get_term_entries(term)
    }
}

pub struct FileEntryState;

#[derive(Clone, Copy)]
pub struct TermEntryState {
    index: usize,
}

pub struct WriterContext<'wctx, S> {
    state: IndexWriter<'wctx>,
    data: Option<S>,
    _marker: PhantomData<S>,
}

impl<'wctx, S> WriterContext<'wctx, S> {
    pub fn new(writer: IndexWriter<'wctx>) -> Self {
        Self {
            state: writer,
            data: None,
            _marker: PhantomData,
        }
    }

    pub fn new_with_data(writer: IndexWriter<'wctx>, data: S) -> Self {
        Self {
            state: writer,
            data: Some(data),
            _marker: PhantomData,
        }
    }
}

impl<'wctx> WriterContext<'wctx, FileEntryState> {
    pub fn entry(
        mut self,
        path: String,
        word_count: usize,
    ) -> WriterContext<'wctx, TermEntryState> {
        let entry = FileEntry::new(path, word_count);
        let index = self.state.insert_file_entry(entry);
        WriterContext::<'wctx, TermEntryState>::new_with_data(self.state, TermEntryState { index })
    }
}

impl<'wctx> WriterContext<'wctx, TermEntryState> {
    pub fn insert_term(&mut self, term: String) {
        self.state.insert_counter(term.clone());

        let word_frequency = unsafe { self.state.get_word_frequency_unchecked(&term) };
        let index = self.data.unwrap().index;

        let entry = TfEntry::new(index, word_frequency);
        self.state.insert_term(term, entry)
    }

    pub fn insert_term_with(&mut self, f: impl FnOnce() -> Token) {
        let term = f().into();
        self.insert_term(term);
    }

    pub fn reset_counter(&mut self) {
        self.state.reset_count()
    }
}

// impl<S> Drop for WriterContext<'_, S> {
//     fn drop(&mut self) {
//         todo!()
//     }
// }

#[derive(Debug)]
pub struct CoreIndex {
    pub store: FileIndex,
    pub index: InvertedIndex,
    pub count: TermCounter,
}

impl CoreIndex {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            store: FileIndex::with_capacity(capacity),
            index: InvertedIndex::with_capacity(capacity),
            count: TermCounter::new(),
        }
    }

    /// Number of indexed documents.
    #[inline]
    pub fn total_docs(&self) -> usize {
        self.store.len()
    }

    #[inline]
    pub fn insert_counter(&mut self, term: String) {
        self.count.insert(term);
    }

    /// Number of documents containing the term.
    pub fn doc_term_frequency(&self, term: &str) -> Option<usize> {
        // todo!()
        match self.get_term_entries(term) {
            Some(entry) => Some(entry.count()),
            None => None,
        }
    }

    fn get_term_entries(&self, term: &str) -> Option<&IdfEntry> {
        self.index.get_term_entries(term)
    }

    /// SAFETY: The caller ensures that the counter is not zero, at least one term is inserted.
    #[inline]
    unsafe fn get_word_frequency_unchecked(&self, term: &str) -> usize {
        unsafe { self.count.get_unchecked(term) }
    }

    /// Provides read access via `IndexReader`
    pub fn reader(&self) -> IndexReader {
        IndexReader {
            store: &self.store,
            index: &self.index,
            count: &self.count,
        }
    }

    /// Provides write access via `indexwriter`
    pub fn writer(&mut self) -> IndexWriter {
        IndexWriter {
            store: &mut self.store,
            index: &mut self.index,
            count: &mut self.count,
        }
    }
}

pub struct IndexReader<'r> {
    store: &'r FileIndex,
    index: &'r InvertedIndex,
    count: &'r TermCounter,
}

impl<'r> IndexReader<'r> {
    /// Number of indexed documents
    #[inline]
    pub fn total_docs(&self) -> usize {
        self.store.len()
    }

    // TODO:
    // - Define the TF-IDF ops in trait
    // - Whether to return Option<T> or concrete type?

    /// Number of documents containing the term
    pub fn doc_term_frequency(&self, term: &str) -> Option<usize> {
        self.get_term_entries(term).map(|entry| entry.count())
    }

    /// Get indexed entries for a term
    pub fn get_term_entries(&self, term: &str) -> Option<&IdfEntry> {
        self.index.get_term_entries(term)
    }

    /// # Panics
    /// If no term exists in the index.
    #[inline]
    pub unsafe fn get_word_frequency_unchecked(&self, term: &str) -> usize {
        self.count.get_unchecked(term)
    }
}

pub struct IndexWriter<'w> {
    store: &'w mut FileIndex,
    index: &'w mut InvertedIndex,
    count: &'w mut TermCounter,
}

impl<'w> IndexWriter<'w> {
    // Insert a term into the term counter
    #[inline]
    pub fn insert_counter(&mut self, term: String) {
        self.count.insert(term);
    }

    // Insert a file entry
    pub fn insert_file_entry(&mut self, entry: FileEntry) -> usize {
        self.store.insert(entry)
    }

    pub fn insert_term(&mut self, term: String, entry: TfEntry) {
        self.index.add_term(term, entry)
    }

    // Reset the term counter
    pub fn reset_count(&mut self) {
        self.count.reset()
    }

    /// # Panics
    /// If no term exists in the index.
    #[inline]
    pub unsafe fn get_word_frequency_unchecked(&self, term: &str) -> usize {
        self.count.get_unchecked(term)
    }
}

// pub struct FileEntryContext<'ctx> {
//     // indexer: &'ctx mut CoreIndex,
//     indexer: &'ctx mut IndexWriter<'ctx>,
// }

// impl<'ctx> FileEntryContext<'ctx> {
//     fn new(indexer: &'ctx mut IndexWriter<'ctx>) -> Self {
//         Self { indexer }
//     }

//     pub fn insert<S: Into<String>>(
//         &'ctx mut self,
//         path: S,
//         word_count: usize,
//     ) -> TermEntryContext<'ctx> {
//         let entry = FileEntry::new(path.into(), word_count);
//         let index = self.insert_inner(entry);
//         TermEntryContext::new(self.indexer, index)
//     }

//     fn insert_inner(&mut self, entry: FileEntry) -> usize {
//         self.indexer.store.insert(entry)
//     }
// }

// pub struct TermEntryContext<'ctx> {
//     indexer: &'ctx mut IndexWriter<'ctx>,
//     file_index: usize,
// }

// impl<'ctx> TermEntryContext<'ctx> {
//     fn new(indexer: &'ctx mut IndexWriter<'ctx>, file_index: usize) -> Self {
//         Self {
//             indexer,
//             file_index,
//         }
//     }

//     pub fn add_term<S: Into<String>>(&mut self, term: S) {
//         let term: String = term.into();

//         self.insert_counter(term.clone());
//         let word_frequency = unsafe { self.indexer.get_word_frequency_unchecked(&term) };
//         let entry = TfEntry::new(self.file_index, word_frequency);

//         self.insert_inner(term, entry);
//     }

//     pub fn add_term_with(&mut self, f: impl FnOnce() -> Token) {
//         let term = f();
//         self.add_term(term);
//     }

//     fn insert_inner(&mut self, term: String, entry: TfEntry) {
//         self.indexer.index.add_term(term, entry);
//     }

//     fn insert_counter(&mut self, term: String) {
//         self.indexer.insert_counter(term);
//     }

//     #[allow(unused)]
//     fn reset_term_counter(&mut self) {
//         self.indexer.count.reset();
//     }
// }

// impl Drop for TermEntryContext<'_> {
//     fn drop(&mut self) {
//         self.indexer.count.reset();
//     }
// }

// pub trait InvertedIndexExt {
//     fn new(capacity: usize) -> Self;
//     fn get_term_entries(&self) -> IdfEntry;
//     fn add_term(&mut self, term: String, tf_entry: TfEntry);
// }

// pub trait FileIndexExt {
//     fn new(capacity: usize) -> Self;
//     fn insert(&mut self, item: FileEntry) -> usize;
//     fn get(&self, index: usize) -> Option<&FileEntry>;
// }

// pub trait TermCounterExt {
//     fn new() -> Self;
//     fn insert(&mut self, key: String) -> Result<(), Err>;
//     fn get(&self, key: &str) -> Option<&Counter<usize>>;
// }

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

    #[inline]
    pub fn get(&self, index: usize) -> Option<&FileEntry> {
        self.inner.get(index)
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

    /// Adds a term to the inverted index with its associated `RefEntry`.    
    #[inline]
    pub fn add_term(&mut self, term: String, tf_entry: TfEntry) {
        // TODO: Track default capacity and threshold.
        self.inner
            .entry_ref(&term)
            .and_modify(|entry| entry.add_entry(RefEntry::new(tf_entry)))
            .or_insert_with(|| {
                let mut set = HashSet::new();
                set.insert(RefEntry::new(tf_entry));
                IdfEntry { entries: set }
            });
    }

    /// Returns an immutable reference to the `IdfEntry` for a given term.
    #[inline]
    pub fn get_term_entries(&self, term: &str) -> Option<&IdfEntry> {
        match self.inner.get(term) {
            Some(entry) => Some(entry),
            None => None,
        }
    }

    /// Number of documents containing the term.
    #[inline]
    pub fn count(&self, term: &str) -> Option<usize> {
        match self.get_term_entries(term) {
            Some(entry) => Some(entry.count()),
            None => None,
        }
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

    /// Number of documents containing the term.
    #[inline]
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Adds a `RefEntry` to this `IdfEntry`.
    pub fn add_entry(&mut self, entry: RefEntry) {
        if let Some(entry) = self.entries.get(&entry) {
            // entry.0.borrow_mut().frequency += 1;
            entry.0.borrow_mut().frequency.increment();
        }

        self.entries.insert(entry);
    }

    /// Retrieves all the entries associated with this `IdfEntry`.
    #[inline]
    pub fn get_entries(&self) -> &HashSet<RefEntry> {
        &self.entries
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
    #[inline]
    pub fn new(entry: TfEntry) -> Self {
        Self(RefCell::new(entry))
    }

    #[inline]
    pub fn increment_frequency(&self) {
        self.0.borrow_mut().increment_frequency()
    }

    #[inline]
    pub fn tf_entry(&self) -> TfEntry {
        *self.0.borrow()
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
    frequency: Counter<usize>,
}

impl TfEntry {
    #[inline]
    pub fn get_frequency(&mut self) -> Counter<usize> {
        self.frequency
    }

    #[inline]
    pub fn get_index(&mut self) -> usize {
        self.index
    }

    #[inline]
    pub fn increment_frequency(&mut self) {
        self.frequency.increment();
    }
}

impl TfEntry {
    pub fn new(index: usize, frequency: usize) -> Self {
        Self {
            index,
            frequency: Counter::new(frequency),
        }
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

    use super::{CoreIndex, FileEntry, FileIndex, InvertedIndex};

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

        let mut indexer = CoreIndex::with_capacity(20);
        let mut writer = indexer.writer();
        writer.insert_counter("t".into());

        let reader = indexer.reader();
        let entries = reader.get_term_entries("t");

        // let mut index = InvertedIndex::with_capacity(INDEX_CAP);
        // let mut file_index = FileIndex::with_capacity(INDEX_CAP);

        // let file_path_index = file_index.insert(FileEntry::new(path, word_count));
        // let mut tf_entry = TfEntry::new(file_path_index, frequency);

        // index.insert(term, tf_entry);

        // let total_doc = 100;
        // let document = "he is good boy".to_string();
        // let term = "boy".to_string();

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
