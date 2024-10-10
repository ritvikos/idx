use std::{
    fmt::{Display, Formatter},
    ops::Deref,
};

use crate::{token::Tokens, tokenizer::Tokenizer};

#[derive(Debug)]
pub struct Document(String);

impl Document {
    #[inline]
    pub fn new(text: String) -> Self {
        Self(text)
    }

    #[inline]
    pub fn tokenize(&self, tokenizer: &mut Tokenizer) -> Tokens {
        tokenizer.tokenize(self.as_ref())
    }
}

impl<T: Into<String>> From<T> for Document {
    fn from(buffer: T) -> Self {
        Self(buffer.into())
    }
}

impl Deref for Document {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Document {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
