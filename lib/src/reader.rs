use std::fmt::Debug;

use crate::core::{IdfEntry, InvertedIndex, Resource, Store, TermCounter};

pub struct IndexReader<'r, R: Clone + Debug> {
    store: &'r Store<R>,
    index: &'r InvertedIndex,
    count: &'r TermCounter,
}

impl<'r, R: Clone + Debug> IndexReader<'r, R> {
    pub fn new(store: &'r Store<R>, index: &'r InvertedIndex, count: &'r TermCounter) -> Self {
        Self {
            store,
            index,
            count,
        }
    }
}

impl<'r, R: Clone + Debug> IndexReader<'r, R> {
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

    #[inline]
    pub fn get_index(&self, index: usize) -> Option<&Resource<R>> {
        self.store.get(index)
    }

    /// # Panics
    /// If no term exists in the index.
    #[inline]
    pub unsafe fn get_word_frequency_unchecked(&self, term: &str) -> usize {
        self.count.get_unchecked(term)
    }
}

pub struct ReaderContext<'rctx, R: Clone + Debug> {
    reader: IndexReader<'rctx, R>,
}

impl<'rctx, R: Clone + Debug> ReaderContext<'rctx, R> {
    pub fn new(reader: IndexReader<'rctx, R>) -> Self {
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
    pub fn get_resource(&self, index: usize) -> Option<R> {
        self.store().get_path(index)
    }

    // Always greater than zero,
    // since empty documents are not indexed.
    #[inline]
    pub fn count(&self, index: usize) -> usize {
        self.store().get(index).unwrap().count()
    }

    #[inline]
    fn inverted_index(&self) -> &InvertedIndex {
        self.reader.index
    }

    #[inline]
    fn store(&self) -> &Store<R> {
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
        self.inverted_index().get_entry_with(term, f)
    }
}
