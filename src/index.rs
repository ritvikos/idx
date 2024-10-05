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

use std::{cell::RefCell, hash::Hash, marker::PhantomData, num::NonZeroUsize};

use hashbrown::{
    hash_map::HashMap,
    hash_set::{HashSet, Iter},
};

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

pub trait Indexer<T> {
    fn new(options: Option<T>) -> Self;
    fn insert(&mut self, path: String, word_count: usize, tokens: &mut Tokens);
    fn get(&self, term: &str);
}

impl Indexer<IndexOptions> for Index {
    fn new(options: Option<IndexOptions>) -> Self {
        match options {
            Some(options) => {
                // TODO: Ensure threshold is less than capacity.
                let capacity = options.capacity;
                let threshold = options.threshold;

                Self {
                    core: CoreIndex::with_capacity(capacity),
                    capacity,
                    threshold,
                }
            }
            None => Self {
                core: CoreIndex::with_capacity(10_000),
                threshold: 9_500,
                capacity: 10_000,
            },
        }
    }

    // TODO:
    // File entry is stored.
    // What if system outage happens at this stage.
    // The file store and inverted index won't sync,
    // ends up with potentially corrupted state.

    // Approach (runtime overhead):
    // Maintain write-ahead logs to re-construct the
    // core index in correct state.
    fn insert(&mut self, path: String, word_count: usize, tokens: &mut Tokens) {
        let writer = self.core.writer();
        let file_entry = WriterContext::<FileEntryState>::new(writer);
        let mut term_entry = file_entry.entry(path, word_count);

        tokens.iter_mut().for_each(|token| {
            term_entry.insert_term_with(|| std::mem::take(token));
        });

        term_entry.reset_counter()
    }

    fn get(&self, term: &str) {
        let reader = self.core.reader();
        let ctx = ReaderContext::new(reader);

        ctx.get_entry_with(term, |idf_entry| {
            debug_assert!(idf_entry.count() > 0);

            idf_entry.iter_with(|ref_entry| {
                let partial = ref_entry.partial_tf();
                let index = partial.get_index();

                // Always greater than zero,
                // since empty documents are not indexed.
                let file_entry = self.store.get(index);
                debug_assert_ne!(file_entry, None);

                // Insertion will occur only if at least one entry exists,
                // so entry is always greater than zero.
                let count = file_entry.unwrap().count();
                debug_assert!(count > 0);

                // TODO: Return `TermFrequencies` here and use `idf` method.
                partial.build(count)
            })
        });
    }
}

pub struct IndexOptions {
    capacity: usize,
    threshold: usize,
}

impl Index {
    // /// Creates a new instance of `Index`
    // pub fn new(capacity: usize, threshold: usize) -> Self {
    //     // TODO: Ensure threshold is less than capacity.

    //     Self {
    //         core: CoreIndex::with_capacity(capacity),
    //         capacity,
    //         threshold,
    //     }
    // }

    /// Number of documents containing the term.
    #[inline]
    pub fn document_frequency(&self, term: &str) -> Option<usize> {
        let reader = self.core.reader();
        let ctx = ReaderContext::new(reader);
        ctx.document_frequency(term)
    }

    /// Number of indexed documents.
    #[inline]
    pub fn total_docs(&self) -> usize {
        let reader = self.core.reader();
        let ctx = ReaderContext::new(reader);
        ctx.total_documents()
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
    writer: IndexWriter<'wctx>,
    data: Option<S>,
    _marker: PhantomData<S>,
}

impl<'wctx, S> WriterContext<'wctx, S> {
    pub fn new(writer: IndexWriter<'wctx>) -> Self {
        Self {
            writer,
            data: None,
            _marker: PhantomData,
        }
    }

    pub fn new_with_data(writer: IndexWriter<'wctx>, data: S) -> Self {
        Self {
            writer,
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
        let index = self.writer.insert_file_entry(entry);
        WriterContext::<'wctx, TermEntryState>::new_with_data(self.writer, TermEntryState { index })
    }
}

impl<'wctx> WriterContext<'wctx, TermEntryState> {
    pub fn insert_term(&mut self, term: String) {
        self.writer.insert_counter(term.clone());

        let word_frequency = unsafe { self.writer.get_word_frequency_unchecked(&term) };
        let index = self.data.unwrap().index;

        let entry = TfEntry::new(index, word_frequency);
        self.writer.insert_term(term, entry)
    }

    pub fn insert_term_with(&mut self, f: impl FnOnce() -> Token) {
        let term = f().into();
        self.insert_term(term);
    }

    #[must_use = "The counter must be reset at the end"]
    pub fn reset_counter(&mut self) {
        self.writer.reset_count()
    }
}

pub struct ReaderContext<'rctx> {
    reader: IndexReader<'rctx>,
}

impl<'rctx> ReaderContext<'rctx> {
    pub fn new(reader: IndexReader<'rctx>) -> Self {
        Self { reader }
    }

    #[inline]
    pub fn total_documents(&self) -> usize {
        self.reader.total_documents()
    }

    #[inline]
    pub fn document_frequency(&self, term: &str) -> Option<usize> {
        self.reader.document_frequency(term)
    }

    #[inline]
    fn index(&self) -> &InvertedIndex {
        self.reader.index
    }

    #[inline]
    fn store(&self) -> &FileIndex {
        self.reader.store
    }

    // low-level function to retrieve `IdfEntry`, if exists.
    #[inline]
    pub fn get_entry(&self, term: &str) -> Option<&IdfEntry> {
        self.reader.get_term_entries(term)
    }

    // low-level function to perform read operations on the entry, if exists.
    #[inline]
    pub fn get_entry_with<O>(&self, term: &str, f: impl FnOnce(&IdfEntry) -> O) -> Option<O> {
        self.index().get_entry_with(term, f)
    }
}

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

    #[inline]
    pub fn insert_counter(&mut self, term: String) {
        self.count.insert(term);
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

    /// Provides write access via `IndexWriter`
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
    pub fn total_documents(&self) -> usize {
        self.store.len()
    }

    // TODO:
    // - Define the TF-IDF ops in trait
    // - Whether to return Option<T> or concrete type?

    /// Number of documents containing the term
    #[inline]
    pub fn document_frequency(&self, term: &str) -> Option<usize> {
        self.index.get_entry_with(term, |entry| entry.count())
    }

    /// Get indexed entries for a term
    #[inline]
    pub fn get_term_entries(&self, term: &str) -> Option<&IdfEntry> {
        self.index.get_term_entries(term)
    }

    // pub fn term_frequencies(&self, term: &str) -> Option<Vec<Tf>> {
    //     self.index.get_entry_with(term, |idf_entry| {
    //         debug_assert!(idf_entry.count() > 0);

    //         idf_entry.iter_with(|ref_entry| {
    //             let partial = ref_entry.partial_tf();
    //             let index = partial.get_index();
    //             // let index = ref_entry.get_index();
    //             // let frequency = ref_entry.get_frequency();

    //             // Always greater than zero,
    //             // since empty documents are not indexed.
    //             let file_entry = self.store.get(index);
    //             debug_assert_ne!(file_entry, None);

    //             // Insertion will occur only if at least one entry exists,
    //             // so entry is always greater than zero.
    //             let count = file_entry.unwrap().count();
    //             debug_assert!(count > 0);

    //             // TODO: Return `TermFrequencies` here and use `idf` method.
    //             partial.build(count)
    //         })
    //     })
    // }

    // pub fn idf(&self, term: &str) -> f32 {
    //     let entry = self.index.get_term_entries(term).unwrap();
    //     let total = self.store.len() as f32;
    //     let count = entry.count() as f32;

    //     let net = total / count;
    //     net.log2()
    // }

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

// TODO: Decide type for `value` field.
#[derive(Debug)]
pub struct Tf {
    index: usize,
    value: f32,
}

impl Tf {
    #[inline]
    pub fn new(index: usize, value: f32) -> Self {
        Self { index, value }
    }

    #[inline]
    pub fn get_index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn get_value(&self) -> f32 {
        self.value
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

    #[inline]
    pub fn get(&self, index: usize) -> Option<&FileEntry> {
        self.inner.get(index)
    }
}

#[derive(Debug, PartialEq, Eq)]
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
            // - The value must not be zero, so empty documents are not indexed.
            count: unsafe { NonZeroUsize::new_unchecked(word_count) },
        }
    }

    pub fn count(&self) -> usize {
        self.count.get()
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

    /// Retrieves the `IdfEntry` associated with the specified `term` and applies
    /// the function `f` to it, returning the result wrapped in `Option`.
    /// If the term is not found, it returns `None`.
    #[inline]
    pub fn get_entry_with<O>(&self, term: &str, f: impl FnOnce(&IdfEntry) -> O) -> Option<O> {
        match self.get_term_entries(term) {
            Some(entry) => Some(f(entry)),
            None => None,
        }
    }
}

pub struct EntriesIter<'a> {
    inner: Iter<'a, RefEntry>,
}

impl<'a> Iterator for EntriesIter<'a> {
    type Item = &'a RefEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
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
            entry.increment_frequency()
        }

        self.entries.insert(entry);
    }

    /// Retrieves all the entries associated with this `IdfEntry`.
    #[inline]
    pub fn get_entries(&self) -> &HashSet<RefEntry> {
        &self.entries
    }

    #[inline]
    pub fn iter(&self) -> EntriesIter {
        EntriesIter {
            inner: self.entries.iter(),
        }
    }

    pub fn iter_with(&self, mut f: impl FnMut(&RefEntry) -> Tf) -> Vec<Tf> {
        self.iter()
            .map(|ref_entry| f(ref_entry))
            .collect::<Vec<Tf>>()
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
    pub fn tf_entry(&self) -> TfEntry {
        *self.0.borrow()
    }

    #[inline]
    pub fn tf_entry_mut(&self) -> TfEntry {
        *self.0.borrow_mut()
    }

    #[inline]
    pub fn get_index(&self) -> usize {
        self.tf_entry().get_index()
    }

    #[inline]
    pub fn get_frequency(&self) -> Counter<usize> {
        self.tf_entry().get_frequency()
    }

    #[inline]
    pub fn increment_frequency(&self) {
        self.tf_entry_mut().increment_frequency()
    }

    #[inline]
    pub fn partial_tf(&self) -> PartialTf {
        let index = self.get_index();
        let frequency = *self.get_frequency() as f32;
        PartialTf::new(index, frequency)
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

pub struct PartialTf {
    index: usize,
    frequency: f32,
}

impl PartialTf {
    #[inline]
    pub fn new(index: usize, frequency: f32) -> Self {
        Self { index, frequency }
    }

    #[inline]
    pub fn get_index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn get_frequency(&self) -> f32 {
        self.frequency
    }

    #[inline]
    #[must_use = "`PartialTf` is lazy. Build it."]
    pub fn build(self, word_count: usize) -> Tf {
        let frequency = self.frequency;
        let index = self.index;
        let word_count = word_count as f32;
        let value = frequency / word_count;

        Tf::new(index, value)
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
