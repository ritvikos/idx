use crate::{normalizer::TextNormalizer, token::Tokens};

#[derive(Clone, Debug, Default)]
pub struct Punctuation;

impl Punctuation {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TextNormalizer for Punctuation {
    fn normalize(&mut self, tokens: &mut Tokens) {
        tokens.iter_mut().for_each(|token| {
            token.inner_mut().retain(|ch| !ch.is_ascii_punctuation());
        });
    }
}
