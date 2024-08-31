use crate::{normalizer::TextNormalizer, tokenizer::Tokens};

#[derive(Debug, Default)]
pub struct Lowercase;

impl Lowercase {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TextNormalizer for Lowercase {
    fn normalize(&mut self, tokens: &mut Tokens) {
        tokens.iter_mut().for_each(|token| {
            let token_mut = token.as_mut();
            token_mut.make_ascii_lowercase();
        })
    }
}

#[derive(Debug, Default)]
pub struct Uppercase;

impl Uppercase {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TextNormalizer for Uppercase {
    fn normalize(&mut self, tokens: &mut Tokens) {
        tokens.iter_mut().for_each(|token| {
            let token_mut = token.as_mut();
            token_mut.make_ascii_uppercase();
        })
    }
}
