use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use crate::{normalizer::TextNormalizer, tokenizer::Tokens};

#[derive(Clone, Debug)]
pub struct Stopwords(Arc<RwLock<HashSet<String>>>);

impl Stopwords {
    pub fn new<I, S>(words: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let set = words.into_iter().map(Into::into).collect::<HashSet<_>>();
        Self(Arc::new(RwLock::new(set)))
    }

    pub fn insert(&mut self, word: String) {
        let mut guard = self.0.write().unwrap();
        guard.insert(word);
    }
}

impl TextNormalizer for Stopwords {
    fn normalize(&mut self, tokens: &mut Tokens) {
        let stopwords = self.0.read().unwrap();
        tokens.retain_mut(|token| {
            let token = token.as_mut();
            token.make_ascii_lowercase();
            !stopwords.contains(token)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        normalizer::{Stopwords, TextNormalizer},
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

    #[test]
    fn test_normalizer_stopwords_dynamic() {
        let mut stopwords = vec![
            "i".to_string(),
            "am".to_string(),
            "are".to_string(),
            "the".to_string(),
        ];

        let mut tokens = tokens![
            "Are", "you", "excited", "about", "the", "new", "project", ",", "or", "am", "I", "the",
            "only", "one", "who", "is", "?",
        ];

        stopwords.push("is".to_string());

        let mut normalizer = Stopwords::new(&stopwords);
        normalizer.normalize(&mut tokens);

        assert_eq!(
            tokens,
            tokens![
                "you", "excited", "about", "new", "project", ",", "or", "only", "one", "who", "?"
            ]
        )
    }
}
