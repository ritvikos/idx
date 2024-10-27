use std::fmt::Debug;

use idx::{document::Document, token::Tokens, tokenizer::Tokenizer};

#[derive(Debug)]
pub struct Descriptor<R: Clone + Debug> {
    document: Document,
    resource: R,
}

impl<R: Clone + Debug> Descriptor<R> {
    #[inline]
    pub fn new(resource: R, document: Document) -> Self {
        Self { resource, document }
    }

    #[inline]
    pub fn inner(&self) -> &Document {
        &self.document
    }

    #[inline]
    pub fn resource(&self) -> R {
        self.resource.clone()
    }

    #[inline]
    pub fn tokenize(&self, tokenizer: &mut Tokenizer) -> Tokens {
        self.document.tokenize(tokenizer)
    }
}
