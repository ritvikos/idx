use crate::{normalizer::TextNormalizer, tokenizer::Tokens};

#[derive(Debug, Default)]
pub struct Punctuation;

impl Punctuation {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TextNormalizer for Punctuation {
    fn normalize(&mut self, tokens: &mut Tokens) {
        tokens.iter_mut().for_each(|token| {
            token.inner().retain(|ch| !ch.is_ascii_punctuation());
        });
    }
}
