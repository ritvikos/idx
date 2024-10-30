use std::{fmt::Debug, marker::PhantomData};

use crate::{core::TfIdf, reader::ReaderContext};

pub trait Ranker<'a, R: Clone + Debug> {
    fn new(reader: &'a ReaderContext<'a, R>) -> Self;
    fn get(&self, term: &str) -> Option<Vec<TfIdf>>;
}

// pub struct Bm25<'a> {
//     reader: &'a ReaderContext<'a>,
// }

// impl<'a> Ranker<'a> for Bm25<'a> {
//     #[inline]
//     fn new(reader: &'a ReaderContext<'a>) -> Self {
//         Self { reader }
//     }

//     fn get(&self, term: &str) -> Option<Vec<TfIdf>> {
//         let total_documents = self.reader.total_documents();
//         let document_frequency = self.reader.document_frequency(term).unwrap();

//         self.reader.get_entry_with(term, |idf_entry| {
//             idf_entry.iter_with(|ref_entry| {
//                 let bm25 = BM25Inner::new(1.5, 0.75);

//                 let idf = bm25.idf(total_documents, document_frequency);

//                 let index = ref_entry.get_index();
//                 let count = self.reader.count(index);
//                 let frequency = *ref_entry.get_frequency();

//                 // FIXME: Handle average word-count, currently hard-coded
//                 let score = bm25.calculate(frequency, count, 3, idf);

//                 TfIdf::new(index, score)
//             })
//         })
//         // .map(Field::from)
//     }
// }

// /// Non-weighted BM25
// pub struct BM25Inner {
//     pub k1: f32,
//     pub b: f32,
// }

// impl BM25Inner {
//     #[inline]
//     pub fn new(k1: f32, b: f32) -> Self {
//         Self { k1, b }
//     }

//     #[inline]
//     pub fn idf(&self, total_documents: usize, document_frequency: usize) -> f32 {
//         ((total_documents as f32 - document_frequency as f32 + 0.5)
//             / (document_frequency as f32 + 0.5)
//             + 1.0)
//             .log10()
//     }

//     #[inline]
//     pub fn calculate(
//         &self,
//         frequency: usize,
//         word_count: usize,
//         avg_word_count: usize,
//         idf: f32,
//     ) -> f32 {
//         let tf = frequency as f32;
//         let num = tf * (self.k1 + 1.0);
//         let denom =
//             tf + self.k1 * (1.0 - self.b + self.b * word_count as f32 / avg_word_count as f32);

//         idf * num / denom
//     }
// }

pub struct TfIdfRanker<'a, R: Clone + Debug> {
    reader: &'a ReaderContext<'a, R>,
    _marker: PhantomData<R>,
}

impl<R: Clone + Debug> TfIdfRanker<'_, R> {
    // FIXME: Need more robust conversion mechanism.
    pub fn tf(&self, frequency: usize, word_count: usize) -> f32 {
        frequency as f32 / word_count as f32
    }

    pub fn idf(&self, total_documents: usize, document_frequency: usize) -> f32 {
        (total_documents as f32 / document_frequency as f32).log10()
    }
}

impl<'a, R: Clone + Debug> Ranker<'a, R> for TfIdfRanker<'a, R> {
    fn new(reader: &'a ReaderContext<'a, R>) -> Self {
        Self {
            reader,
            _marker: PhantomData,
        }
    }

    fn get(&self, term: &str) -> Option<Vec<TfIdf>> {
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

                    TfIdf::new(index, tfidf)
                })
                .collect::<Vec<_>>()
        })
    }
}
