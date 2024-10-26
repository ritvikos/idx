//! The [`CoreIndex`] is the core component that handles indexing operations.
//! Internally, it manages inverted-index, file-index, term-counter.
//!
//! Inverted Index
//! The [`InvertedIndex`] handles the core inverted index data structure and
//! exposes methods to perform operations.

use crate::{
    core::{FileIndex, InvertedIndex, TermCounter, TfIdf},
    rank::{Ranker, TfIdfRanker},
    reader::{IndexReader, ReaderContext},
    token::Tokens,
    writer::{FileEntryState, IndexWriter, WriterContext},
};

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
    pub core: CoreIndex,
    pub capacity: usize,
    pub threshold: usize,
}

pub trait Indexer {
    fn new(capacity: usize, threshold: usize) -> Self;
    fn insert(&mut self, path: String, word_count: usize, tokens: &mut Tokens);
    fn get_score(&self, term: &str) -> Option<Vec<TfIdf>>;

    // TODO: return resource instead.
    fn get_resource(&self, index: usize) -> Option<String>;
}

impl Indexer for Index {
    /// Creates a new instance of `Index`
    fn new(capacity: usize, threshold: usize) -> Self {
        // TODO: Ensure threshold is less than capacity.

        Self {
            core: CoreIndex::with_capacity(capacity),
            capacity,
            threshold,
        }
    }

    // TODO:
    // File entry is stored.
    // What if system outage happens at this stage.
    // The file store and inverted index won't sync,
    // ends up with potentially corrupted state.

    // Approach 1 (runtime overhead):
    // Maintain write-ahead logs to re-construct the
    // core index in correct state.

    // Approach 2:
    // On startup, validate for such scenarios and handle,
    // before re-constructing the in-memory structure.
    fn insert(&mut self, path: String, word_count: usize, tokens: &mut Tokens) {
        let writer = self.core.writer();
        let file_entry = WriterContext::<FileEntryState>::new(writer);
        let mut term_entry = file_entry.entry(path, word_count);

        tokens.for_each_mut(|token| {
            term_entry.insert_term_with(|| std::mem::take(token));
        });

        term_entry.reset_counter()
    }

    fn get_score(&self, term: &str) -> Option<Vec<TfIdf>> {
        let reader = self.core.reader();
        let ctx = ReaderContext::new(reader);

        let ranker = TfIdfRanker::new(&ctx);
        ranker.get(term)
    }

    fn get_resource(&self, index: usize) -> Option<String> {
        let reader = self.core.reader();
        let ctx = ReaderContext::new(reader);

        ctx.get_resource(index)
    }
}

#[derive(Debug)]
pub struct CoreIndex {
    store: FileIndex,
    index: InvertedIndex,
    count: TermCounter,
}

impl CoreIndex {
    /// Creates a new instance of [`CoreIndex`]
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            store: FileIndex::with_capacity(capacity),
            index: InvertedIndex::with_capacity(capacity),
            count: TermCounter::new(),
        }
    }

    /// The `CoreIndex` never interacts with external environment
    /// directly for READ operations.
    ///
    /// [`IndexReader`] provides READ access to the index.
    ///
    /// # See Also
    ///
    /// - [`IndexWriter`]: Provides WRITE access to the index.
    pub fn reader(&self) -> IndexReader {
        IndexReader::new(&self.store, &self.index, &self.count)
    }

    /// The `CoreIndex` never interacts with external environment
    /// directly for WRITE operations.
    ///
    /// [`IndexWriter`] provides WRITE access to the index.
    ///
    /// # See Also
    ///
    /// - [`IndexReader`]: Provides READ access to the index.
    pub fn writer(&mut self) -> IndexWriter {
        IndexWriter::new(&mut self.store, &mut self.index, &mut self.count)
    }
}
