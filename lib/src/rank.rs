use crate::{
    core::{Field, TfIdf},
    index::CoreIndex,
    reader::ReaderContext,
};

pub trait Ranker {
    fn get(&self, term: &str) -> Option<Field>;
}

// FIXME: Handle average word-count, currently hard-coded
pub struct Bm25<'a> {
    index: &'a CoreIndex,
}

impl<'a> Bm25<'a> {
    #[inline]
    pub fn new(index: &'a CoreIndex) -> Self {
        Self { index }
    }
}

impl Ranker for Bm25<'_> {
    fn get(&self, term: &str) -> Option<Field> {
        let reader = self.index.reader();
        let ctx = ReaderContext::new(reader);

        ctx.get_entry_with(term, |idf_entry| {
            idf_entry.iter_with(|ref_entry| {
                let bm25 = BM25Inner::new(1.5, 0.75);

                let total_documents = ctx.total_documents();
                let document_frequency = ctx.document_frequency(term).unwrap();
                let idf = bm25.idf(total_documents, document_frequency);

                let index = ref_entry.get_index();
                let count = ctx.count(index);
                let frequency = *ref_entry.get_frequency();
                let score = bm25.calculate(frequency, count, 3, idf);

                TfIdf::new(index, score)
            })
        })
        .map(Field::from)
    }
}

/// Non-weighted BM25
pub struct BM25Inner {
    pub k1: f32,
    pub b: f32,
}

impl BM25Inner {
    #[inline]
    pub fn new(k1: f32, b: f32) -> Self {
        Self { k1, b }
    }

    #[inline]
    pub fn idf(&self, total_documents: usize, document_frequency: usize) -> f32 {
        ((total_documents as f32 - document_frequency as f32 + 0.5)
            / (document_frequency as f32 + 0.5)
            + 1.0)
            .log10()
    }

    #[inline]
    pub fn calculate(
        &self,
        frequency: usize,
        word_count: usize,
        avg_word_count: usize,
        idf: f32,
    ) -> f32 {
        let tf = frequency as f32;
        let num = tf * (self.k1 + 1.0);
        let denom =
            tf + self.k1 * (1.0 - self.b + self.b * word_count as f32 / avg_word_count as f32);

        idf * num / denom
    }
}
