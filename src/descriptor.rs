use crate::{
    document::Document,
    token::{TokenVec, Tokens},
    tokenizer::Tokenizer,
};

#[derive(Debug)]
pub struct Descriptor {
    document: Document,
    path: String,
}

impl Descriptor {
    #[inline]
    pub fn new(path: String, document: Document) -> Self {
        Self { path, document }
    }

    #[inline]
    pub fn document(&self) -> &Document {
        &self.document
    }

    #[inline]
    pub fn path(&self) -> String {
        self.path.clone()
    }

    #[inline]
    pub fn path_ref(&self) -> &str {
        &self.path
    }

    #[inline]
    pub fn tokenize(&self, tokenizer: &mut Tokenizer) -> Tokens {
        TokenVec::from(self.document.tokenize(tokenizer))
    }

    // #[inline]
    // pub fn word_count(&self) -> usize {
    //     self.document.word_count()
    // }
}
