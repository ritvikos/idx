use crate::{descriptor::Descriptor, query::Query};

use idx::{
    core::Collection,
    index::{Index, Indexer},
    normalizer::NormalizerPipeline,
    token::Tokens,
    tokenizer::Tokenizer,
};

pub trait Engine {
    fn index(&mut self, descriptor: Descriptor);
    fn search(&self, query: Query);
}

// TODO: Partition
#[derive(Debug)]
pub struct IdxFacade<I: Indexer = Index> {
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

    pub fn insert(&mut self, descriptor: Descriptor) {
        let mut tokens = descriptor.tokenize(&mut self.tokenizer);
        let path = descriptor.path();
        let word_count = tokens.count();

        if !self.pipeline.is_empty() {
            self.pipeline.run(&mut tokens);
        }

        self.insert_inner(path, word_count, &mut tokens);
    }

    fn insert_inner<S: Into<String>>(&mut self, path: S, word_count: usize, tokens: &mut Tokens) {
        self.index.insert(path.into(), word_count, tokens);
    }

    pub fn get(&self, query: Query) -> Collection {
        let mut tokenizer = self.tokenizer.clone();
        let mut pipeline = self.pipeline.clone();

        let mut tokens = query.tokenize(&mut tokenizer);

        if !self.pipeline.is_empty() {
            pipeline.run(&mut tokens);
        }

        let capacity = tokens.count();
        let mut collection = Collection::with_capacity(capacity);

        tokens.for_each_mut(|token| {
            self.index.get(token).map(|field| collection.insert(field));
        });

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
        ["the cat sat on the mat", "the cat sat", "the dog barked"]
            .iter()
            .map(|document| document.to_string())
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

        let mut engine: IdxFacade<Index> = IdxFacade::new(10, 30, tokenizer.clone(), pipeline);

        for (idx, document) in corpus.iter().enumerate() {
            let descriptor = Descriptor::new(format!("path_{}", idx), document.into());
            engine.insert(descriptor);
        }

        // let target = "foxes"; // expected: 7
        let target = "cat dog sat";
        // let term_doc_count = engine.document_frequency(&target);
        // let total_docs = engine.total_docs(); // expected: 20

        let query = Query::new(target);
        let tf = engine.get(query);

        println!("engine: {engine:#?}");
        // println!("term_doc_count: {term_doc_count:?}");
        // println!("total_docs: {total_docs}");
        println!("tf: {tf:#?}");
    }
}
