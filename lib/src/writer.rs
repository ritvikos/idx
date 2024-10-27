use std::{fmt::Debug, marker::PhantomData};

use crate::{
    core::{InvertedIndex, Resource, Store, TermCounter, TfEntry},
    token::Token,
};

pub struct IndexWriter<'w, R: Clone + Debug> {
    store: &'w mut Store<R>,
    index: &'w mut InvertedIndex,
    count: &'w mut TermCounter,
}

impl<'w, R: Clone + Debug> IndexWriter<'w, R> {
    pub fn new(
        store: &'w mut Store<R>,
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

impl<'w, R: Clone + Debug> IndexWriter<'w, R> {
    // Insert a term into the term counter
    #[inline]
    pub fn insert_counter(&mut self, term: String) {
        self.count.insert(term);
    }

    // Insert a file entry
    pub fn insert_resource(&mut self, entry: Resource<R>) -> usize {
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

pub struct ResourceState;

#[derive(Clone, Copy)]
pub struct TermEntryState {
    index: usize,
}

pub struct WriterContext<'wctx, S, R: Clone + Debug> {
    writer: IndexWriter<'wctx, R>,
    data: Option<S>,
    _marker: PhantomData<S>,
}

impl<'wctx, S, R: Clone + Debug> WriterContext<'wctx, S, R> {
    pub fn new(writer: IndexWriter<'wctx, R>) -> Self {
        Self {
            writer,
            data: None,
            _marker: PhantomData,
        }
    }

    pub fn new_with_data(writer: IndexWriter<'wctx, R>, data: S) -> Self {
        Self {
            writer,
            data: Some(data),
            _marker: PhantomData,
        }
    }
}

impl<'wctx, R: Clone + Debug> WriterContext<'wctx, ResourceState, R> {
    pub fn entry(
        mut self,
        resource: R,
        word_count: usize,
    ) -> WriterContext<'wctx, TermEntryState, R> {
        let entry = Resource::new(resource, word_count);
        let index = self.writer.insert_resource(entry);
        WriterContext::<'wctx, TermEntryState, R>::new_with_data(
            self.writer,
            TermEntryState { index },
        )
    }
}

impl<'wctx, R: Clone + Debug> WriterContext<'wctx, TermEntryState, R> {
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
