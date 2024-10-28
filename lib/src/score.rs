use std::{fmt::Debug, marker::PhantomData};

use crate::{reader::ReaderContext, token::Token};

pub trait Score<'a> {
    type K;
    type V;
    type R: Clone + Debug;

    fn new(strategy: &'a ReaderContext<'a, Self::R>) -> Self;
    fn score(&self, term: &str) -> Option<Vec<(Self::K, Self::V)>>;
}

pub struct Scorer<'a, S: Score<'a>> {
    inner: S,
    _marker: PhantomData<&'a S>,
}

impl<'a, S: Score<'a>> Scorer<'a, S> {
    pub fn new(strategy: S) -> Self {
        Self {
            inner: strategy,
            _marker: PhantomData,
        }
    }

    pub fn score(&self, term: &str) -> Option<Vec<(S::K, S::V)>> {
        self.inner.score(term)
    }

    pub fn score_and_apply<F>(&mut self, mut f: F, tokens: impl IntoIterator<Item = Token>)
    where
        F: FnMut(Vec<(S::K, S::V)>),
    {
        tokens.into_iter().for_each(|token| {
            if let Some(score) = self.inner.score(&token) {
                f(score)
            }
        });
    }

    pub fn from_tokens(
        &self,
        tokens: impl Iterator<Item = Token>,
    ) -> Vec<Option<Vec<(S::K, S::V)>>> {
        tokens
            .map(|token| self.inner.score(&token))
            .collect::<Vec<Option<Vec<(S::K, S::V)>>>>()
    }
}

pub struct TfIdfScorer<'a, R: Clone + Debug> {
    reader: &'a ReaderContext<'a, R>,
}

impl<R: Clone + Debug> TfIdfScorer<'_, R> {
    // FIXME: Need more robust conversion mechanism.
    pub fn tf(&self, frequency: usize, word_count: usize) -> f32 {
        frequency as f32 / word_count as f32
    }

    pub fn idf(&self, total_documents: usize, document_frequency: usize) -> f32 {
        (total_documents as f32 / document_frequency as f32).log10()
    }
}

impl<'a, R: Clone + Debug> Score<'a> for TfIdfScorer<'a, R> {
    type K = usize;
    type V = f32;
    type R = R;

    fn new(reader: &'a ReaderContext<'a, R>) -> Self {
        Self { reader }
    }

    fn score(&self, term: &str) -> Option<Vec<(Self::K, Self::V)>> {
        let total_documents = self.reader.total_documents();

        self.reader.get_entry_with(term, |idf_entry| {
            debug_assert!(idf_entry.count() > 0);
            let document_frequency = idf_entry.count();

            idf_entry
                .iter()
                .map(|ref_entry| {
                    let index = ref_entry.get_index();
                    let frequency = *ref_entry.get_frequency();

                    // Always greater than zero, empty documents are not indexed.
                    let count = self.reader.count(index);
                    debug_assert!(count > 0);

                    let tf = self.tf(frequency, count);
                    let idf = self.idf(total_documents, document_frequency);
                    let tfidf = tf * idf;
                    (index, tfidf)
                })
                .collect::<Vec<_>>()
        })
    }
}
