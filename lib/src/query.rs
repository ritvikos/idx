use std::ops::Deref;

use crate::{token::Tokens, tokenizer::Tokenizer};

#[derive(Debug)]
pub struct Query<'a>(&'a str);

// TODO:
// - phrase match (exact and proximity)
// - boolean match
// - fuzzy search
// - faceted search
impl<'a> Query<'a> {
    #[inline]
    pub fn new(value: &'a str) -> Self {
        Self(value)
    }

    #[inline]
    pub fn tokenize(&self, tokenizer: &mut Tokenizer) -> Tokens {
        tokenizer.tokenize(&self.0)
    }
}

impl<'a> Deref for Query<'a> {
    type Target = &'a str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Query<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}
