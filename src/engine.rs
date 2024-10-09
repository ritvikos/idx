use crate::descriptor::Descriptor;

use idx::{
    core::Tf,
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
pub struct IdxFacade {
    pub index: Index,
    pub tokenizer: Tokenizer,
    pub pipeline: NormalizerPipeline,
}

impl IdxFacade {
    pub fn new(
        capacity: usize,
        threshold: usize,
        tokenizer: Tokenizer,
        pipeline: NormalizerPipeline,
    ) -> Self {
        Self {
            index: Index::new(capacity, threshold),
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

    // pub fn get(&self, query: Query) {
    //     let mut tokenizer = self.tokenizer.clone();

    //     let tokens = query.tokenize(&mut tokenizer);
    //     let word_count = tokens.count();

    //     let term_doc_count = self.term_doc_count(query.as_ref());
    //     let total_docs = self.total_docs();
    // }

    // FIXME: tf
    pub fn get(&self, query: Query) -> Option<Vec<Tf>> {
        let mut tokenizer = self.tokenizer.clone();
        let mut pipeline = self.pipeline.clone();

        let mut tokens = query.tokenize(&mut tokenizer);

        if !self.pipeline.is_empty() {
            pipeline.run(&mut tokens);
        }

        // let word_count = tokens.count();
        let term = query.0;
        self.index.get(&term)
    }

    /// Number of documents containing the term.
    #[inline]
    pub fn document_frequency(&self, term: &str) -> Option<usize> {
        self.index.document_frequency(term)
    }

    /// Total number of indexed documents.
    #[inline]
    pub fn total_docs(&self) -> usize {
        self.index.total_docs()
    }
}

#[derive(Debug)]
pub struct SearchContext<'ctx> {
    index: &'ctx Index,
    tokenizer: Tokenizer,
    pipeline: &'ctx NormalizerPipeline,
}

impl<'ctx> SearchContext<'ctx> {
    #[inline]
    pub fn new(
        index: &'ctx Index,
        tokenizer: Tokenizer,
        pipeline: &'ctx NormalizerPipeline,
    ) -> Self {
        Self {
            index,
            tokenizer,
            pipeline,
        }
    }

    /// Number of documents containing the term.
    #[inline]
    pub fn document_frequency(&self, term: &str) -> Option<usize> {
        self.index.document_frequency(term)
    }

    /// Total number of indexed documents
    #[inline]
    pub fn total_docs(&self) -> usize {
        self.index.total_docs()
    }

    pub fn search(&self) {
        // tf: Number of times term t appears in document d / Total number of terms in document
        // idf: total number of documents / number of documents containing term
    }
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

impl<T: Into<String>> From<T> for Query {
    #[inline]
    fn from(value: T) -> Self {
        Query(value.into())
    }
}

impl AsRef<str> for Query {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}
