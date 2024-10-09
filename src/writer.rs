use std::marker::PhantomData;

use crate::{
    core::{FileEntry, FileIndex, InvertedIndex, TfEntry},
    map::TermCounter,
    token::Token,
};

pub struct IndexWriter<'w> {
    store: &'w mut FileIndex,
    index: &'w mut InvertedIndex,
    count: &'w mut TermCounter,
}

impl<'w> IndexWriter<'w> {
    pub fn new(
        store: &'w mut FileIndex,
        index: &'w mut InvertedIndex,
        count: &'w mut TermCounter,
    ) -> Self {
        Self {
            store,
            index,
            count,
        }
    }
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

    pub fn reset_counter(&mut self) {
        self.writer.reset_count()
    }
}
