// extern crate hashbrown;

// use hashbrown::HashMap;

use crate::{
    descriptor::Descriptor,
    index::Indexer,
    normalizer::NormalizerPipeline,
    tokenizer::{Tokenizer, Tokens},
};

// TODO: Partition
#[derive(Debug)]
pub struct Coordinator {
    pub indexer: Indexer,
    pub tokenizer: Tokenizer,
    pub pipeline: NormalizerPipeline,
}

impl Coordinator {
    pub fn new(
        capacity: usize,
        threshold: usize,
        tokenizer: Tokenizer,
        pipeline: NormalizerPipeline,
    ) -> Self {
        Self {
            indexer: Indexer::new(capacity, threshold),
            tokenizer,
            pipeline,
        }
    }

    pub fn insert(&mut self, descriptor: Descriptor) {
        let mut tokens = descriptor.tokenize(&mut self.tokenizer);
        let path = descriptor.path();
        let word_count = tokens.len();

        if !self.pipeline.is_empty() {
            self.pipeline.run(&mut tokens);
        }

        self.insert_inner(path, word_count, &mut tokens);
    }

    fn insert_inner<S: Into<String>>(&mut self, path: S, word_count: usize, tokens: &mut Tokens) {
        self.indexer.insert(path.into(), word_count, tokens);
    }

    pub fn get(&self, query: Query) {}
}

#[derive(Debug)]
pub struct Query(String);

impl Query {
    #[inline]
    pub fn new(value: String) -> Self {
        Self(value)
    }

    #[inline]
    pub fn tokenize(&self, tokenizer: &mut Tokenizer) -> Tokens {
        tokenizer.tokenize(&self.0)
    }
}

impl From<String> for Query {
    #[inline]
    fn from(value: String) -> Self {
        Query(value)
    }
}
