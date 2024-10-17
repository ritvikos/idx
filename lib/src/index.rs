//! The [`CoreIndex`] is the core component that handles indexing operations.
//! Internally, it manages inverted-index, file-index, term-counter.
//!
//! Inverted Index
//! The [`InvertedIndex`] handles the core inverted index data structure and
//! exposes methods to perform operations.

use crate::{
    core::{FileIndex, InvertedIndex, TermCounter, TfIdf},
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
    fn get(&self, term: &str) -> Option<Vec<TfIdf>>;
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

    // Approach (runtime overhead):
    // Maintain write-ahead logs to re-construct the
    // core index in correct state.
    fn insert(&mut self, path: String, word_count: usize, tokens: &mut Tokens) {
        let writer = self.core.writer();
        let file_entry = WriterContext::<FileEntryState>::new(writer);
        let mut term_entry = file_entry.entry(path, word_count);

        tokens.for_each_mut(|token| {
            term_entry.insert_term_with(|| std::mem::take(token));
        });

        term_entry.reset_counter()
    }

    fn get(&self, term: &str) -> Option<Vec<TfIdf>> {
        let reader = self.core.reader();
        let ctx = ReaderContext::new(reader);

        ctx.get_entry_with(term, |idf_entry| {
            debug_assert!(idf_entry.count() > 0);

            idf_entry.iter_with(|ref_entry| {
                let index = ref_entry.get_index();
                let frequency = *ref_entry.get_frequency();
                let count = ctx.count(index);

                // Always greater than zero, empty documents are not indexed.
                debug_assert!(count > 0);

                let total_documents = ctx.total_documents();
                let document_frequency = idf_entry.count();

                let tf = self.tf(frequency, count);
                let idf = self.idf(total_documents, document_frequency);
                let tfidf = tf * idf;

                TfIdf::new(index, tfidf)
            })
        })
    }
}

impl Index {
    // FIXME: Need more robust conversion mechanism.
    pub fn tf(&self, frequency: usize, word_count: usize) -> f32 {
        frequency as f32 / word_count as f32
    }

    pub fn idf(&self, total_documents: usize, document_frequency: usize) -> f32 {
        (total_documents as f32 / document_frequency as f32).log10()
    }

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
