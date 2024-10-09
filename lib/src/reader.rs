use crate::{
    core::{FileEntry, FileIndex, IdfEntry, InvertedIndex},
    map::TermCounter,
};

pub struct IndexReader<'r> {
    store: &'r FileIndex,
    index: &'r InvertedIndex,
    count: &'r TermCounter,
}

impl<'r> IndexReader<'r> {
    pub fn new(store: &'r FileIndex, index: &'r InvertedIndex, count: &'r TermCounter) -> Self {
        Self {
            store,
            index,
            count,
        }
    }
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

    #[inline]
    pub fn get_index(&self, index: usize) -> Option<&FileEntry> {
        self.store.get(index)
    }

    /// # Panics
    /// If no term exists in the index.
    #[inline]
    pub unsafe fn get_word_frequency_unchecked(&self, term: &str) -> usize {
        self.count.get_unchecked(term)
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
        self.inverted_index().get_entry_with(term, f)
    }
}
