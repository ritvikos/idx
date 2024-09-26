use std::{
    fmt::{Display, Formatter},
    ops::{Deref, DerefMut},
};

use crate::{
    token::{Token, Tokens},
    tokenizer::Tokenizer,
};

#[derive(Debug)]
pub struct Document(String);

impl Document {
    #[inline]
    pub fn new(text: String) -> Self {
        Self(text)
    }

    #[inline]
    pub fn inner(&self) -> &String {
        &self.0
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut String {
        &mut self.0
    }

    #[inline]
    pub fn tokenize(&self, tokenizer: &mut Tokenizer) -> Tokens {
        tokenizer.tokenize(self.inner())
    }
}

impl From<String> for Document {
    #[inline]
    fn from(buffer: String) -> Self {
        Document(buffer)
    }
}

impl Deref for Document {
    type Target = String;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl DerefMut for Document {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
