//! The [`CoreIndex`] is the core component that handles indexing operations.
//! Internally, it manages inverted-index, file-index, term-counter.
//!
//! Inverted Index
//! The [`InvertedIndex`] handles the core inverted index data structure and
//! exposes methods to perform operations.

use std::fmt::Debug;

use crate::{
    core::{InvertedIndex, Store, TermCounter},
    reader::{IndexReader, ReaderContext},
    token::Tokens,
    writer::{IndexWriter, ResourceState, WriterContext},
};

pub trait Indexer {
    type R: Clone + Debug;

    fn new(capacity: usize, threshold: usize) -> Self;
    fn insert(&mut self, resource: Self::R, word_count: usize, tokens: &mut Tokens);
    fn get(&self, index: usize) -> Option<Self::R>;
    fn reader(&self) -> ReaderContext<'_, Self::R>;
}

/// # Indexer
///
/// Currently, a single-threaded data structure implementation that internally utilizes
/// multithreading and SIMD for data-level parallelism,
/// optimizing throughput and tail latency.
///
/// The current strategy utilizes a single-threaded, thread-local indexer
/// and perform a merge operation to generate a global index view.
#[derive(Debug)]
pub struct Index<R: Clone + Debug> {
    pub core: CoreIndex<R>,
    pub capacity: usize,
    pub threshold: usize,
}

impl<R: Clone + Debug> Indexer for Index<R> {
    type R = R;

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
    fn insert(&mut self, resource: R, word_count: usize, tokens: &mut Tokens) {
        let writer = self.core.writer();
        let resource_entry = WriterContext::<ResourceState, R>::new(writer);
        let mut term_entry = resource_entry.entry(resource, word_count);

        tokens.for_each_mut(|token| {
            term_entry.insert_term_with(|| std::mem::take(token));
        });

        term_entry.reset_counter()
    }

    fn get(&self, index: usize) -> Option<R> {
        let ctx = self.reader();
        ctx.get_resource(index)
    }

    fn reader(&self) -> ReaderContext<'_, Self::R> {
        let reader = self.core.reader();
        ReaderContext::new(reader)
    }
}

#[derive(Debug)]
pub struct CoreIndex<R: Clone + Debug> {
    store: Store<R>,
    index: InvertedIndex,
    count: TermCounter,
}

impl<R: Clone + Debug> CoreIndex<R> {
    /// Creates a new instance of [`CoreIndex`]
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            store: Store::with_capacity(capacity),
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
    pub fn reader(&self) -> IndexReader<R> {
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
    pub fn writer(&mut self) -> IndexWriter<R> {
        IndexWriter::new(&mut self.store, &mut self.index, &mut self.count)
    }
}
