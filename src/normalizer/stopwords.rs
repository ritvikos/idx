use std::collections::HashSet;

use crate::{normalizer::TextNormalizer, tokenizer::Tokens};

#[macro_export]
macro_rules! stopwords {
    [$($word:expr),* $(,)?] => {{
        static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();

        SET.get_or_init(|| {
            let words = [$($word),*];
            words.iter().copied().collect::<HashSet<&str>>()
        })
    }};
}

#[derive(Debug)]
pub struct Stopwords(HashSet<&'static str>);

impl Stopwords {
    pub fn new<const N: usize>(words: [&'static str; N]) -> Self {
        let set = words.iter().copied().collect::<HashSet<_>>();
        Self(set)
    }

    pub fn from_macro(set: HashSet<&'static str>) -> Self {
        Self(set)
    }
}

impl TextNormalizer for Stopwords {
    fn normalize(&mut self, tokens: &mut Tokens) {
        tokens.retain_mut(|token| {
            let token = token.as_mut();
            token.make_ascii_lowercase();
            !self.0.contains(token)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        normalizer::{Stopwords, TextNormalizer},
        tokenizer::Token,
        tokens,
    };

    #[test]
    fn test_normalizer_stopwords() {
        const WITH_STOPWORDS: [&str; 3] = ["the", "and", "in"];

        let mut tokens = tokens!["the", "cat", "in", "the", "hat", "and", "bat"];
        let mut normalizer = Stopwords::new(WITH_STOPWORDS);

        normalizer.normalize(&mut tokens);

        assert_eq!(tokens, tokens!["cat", "hat", "bat"]);
    }

    #[test]
    fn test_normalizer_stopwords_none() {
        const NO_STOPWORDS: [&str; 0] = [];

        let mut tokens = tokens!["one", "two", "three"];
        let mut normalizer = Stopwords::new(NO_STOPWORDS);

        normalizer.normalize(&mut tokens);

        assert_eq!(tokens, tokens!["one", "two", "three"]);
    }

    #[test]
    fn test_normalizer_stopwords_tokenless() {
        const STOPWORDS_EMPTY: [&str; 3] = ["the", "and", "in"];

        let mut tokens = tokens![];
        let mut normalizer = Stopwords::new(STOPWORDS_EMPTY);

        normalizer.normalize(&mut tokens);

        assert_eq!(tokens, tokens![]);
    }

    #[test]
    fn test_normalizer_stopwords_unicode() {
        const STOPWORDS_UNICODE: [&str; 2] = ["naïve", "élève"];

        let mut tokens = tokens!["naïve", "élève", "école"];
        let mut normalizer = Stopwords::new(STOPWORDS_UNICODE);

        normalizer.normalize(&mut tokens);

        assert_eq!(tokens, tokens!["école"]);
    }
}
