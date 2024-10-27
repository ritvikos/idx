use std::fmt::Debug;

use crate::{descriptor::Descriptor, query::Query};

use idx::{
    aggregate::{Aggregator, HashAggregator},
    core::Collection,
    index::Indexer,
    normalizer::NormalizerPipeline,
    tokenizer::Tokenizer,
};

#[derive(Debug)]
pub struct IdxFacade<I: Indexer> {
    pub index: I,
    pub tokenizer: Tokenizer,
    pub pipeline: NormalizerPipeline,
}

impl<I: Indexer> IdxFacade<I> {
    pub fn new(
        capacity: usize,
        threshold: usize,
        tokenizer: Tokenizer,
        pipeline: NormalizerPipeline,
    ) -> Self {
        Self {
            index: Indexer::new(capacity, threshold),
            tokenizer,
            pipeline,
        }
    }

    pub fn insert(&mut self, descriptor: Descriptor<<I as Indexer>::R>) {
        let mut tokens = descriptor.tokenize(&mut self.tokenizer);
        let resource = descriptor.resource();
        let word_count = tokens.count();

        if !self.pipeline.is_empty() {
            self.pipeline.run(&mut tokens);
        }

        self.index.insert(resource, word_count, &mut tokens);
    }

    pub fn get(&self, query: Query) -> Collection {
        let mut tokenizer = self.tokenizer.clone();
        let mut pipeline = self.pipeline.clone();

        let mut tokens = query.tokenize(&mut tokenizer);

        if !self.pipeline.is_empty() {
            pipeline.run(&mut tokens);
        }

        let capacity = tokens.count();
        let collection = Collection::with_capacity(capacity);

        let hash_based = HashAggregator::new();
        let mut aggregator = Aggregator::new(hash_based);

        tokens.for_each_mut(|token| {
            if let Some(fields) = self.index.get_score(token) {
                for field in fields {
                    aggregator.insert(field.get_index(), field.get_score());
                }
            }
        });

        aggregator.iter().for_each(|(index, score)| {
            let resource = self.index.get_resource(*index).unwrap();
            println!("{index} {score}: {resource:?}");
        });

        // temporary
        collection
    }
}

#[cfg(test)]
mod tests {
    use idx::{
        index::Index,
        normalizer::{case::Lowercase, punctuation::Punctuation, NormalizerPipeline, Stopwords},
        tokenizer::{Standard, Tokenizer},
    };

    use crate::{
        descriptor::Descriptor,
        engine::{IdxFacade, Query},
    };

    fn tiny_test_corpus() -> Vec<String> {
        [
            "the cat sat on the mat",
            "the cat sat",
            "the dog barked",
            "penguin is a nice animal",
        ]
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
    }

    #[test]
    fn test_indexer_and_engine() {
        let corpus = tiny_test_corpus();
        let tokenizer = Tokenizer::Standard(Standard::new());

        let mut pipeline = NormalizerPipeline::new();
        pipeline.insert(Box::new(Lowercase::new()));
        pipeline.insert(Box::new(Punctuation::new()));
        pipeline.insert(Box::new(
            Stopwords::load("assets/stopwords/en.txt").unwrap(),
        ));

        let mut engine: IdxFacade<Index<String>> =
            IdxFacade::new(10, 30, tokenizer.clone(), pipeline);

        for document in corpus {
            let descriptor = Descriptor::new(document.clone(), document.into());
            engine.insert(descriptor);
        }

        // let target = "foxes"; // expected: 7
        let target = "cat sat";
        // let term_doc_count = engine.document_frequency(&target);
        // let total_docs = engine.total_docs(); // expected: 20

        let query = Query::new(target);
        let collection = engine.get(query);
        println!("collection: {collection:?}");

        // println!("engine: {engine:#?}");
        // println!("term_doc_count: {term_doc_count:?}");
        // println!("total_docs: {total_docs}");
        // println!("tf: {tf:#?}");
    }
}
