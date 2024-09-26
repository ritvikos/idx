use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::{Arc, RwLock},
};

use crate::{
    normalizer::TextNormalizer,
    token::{Token, Tokens},
};

#[derive(Clone, Debug)]
pub struct TokenReplacer<V>
where
    V: 'static + Clone + Debug + Display + Into<String> + Send + Sync,
{
    pairs: Arc<RwLock<HashMap<String, V>>>,
}

impl<V> TokenReplacer<V>
where
    V: 'static + Clone + Debug + Display + Into<String> + Send + Sync,
{
    pub fn new(pairs: HashMap<String, V>) -> Self {
        Self {
            pairs: Arc::new(RwLock::new(pairs)),
        }
    }

    pub fn insert(&mut self, key: String, value: V) {
        let mut guard = self.pairs.write().unwrap();
        guard.insert(key, value).unwrap();
    }

    pub fn remove(&mut self, key: &str) {
        let mut guard = self.pairs.write().unwrap();
        guard.remove(key);
    }
}

impl<V> TextNormalizer for TokenReplacer<V>
where
    V: 'static + Clone + Debug + Display + Into<String> + Send + Sync,
{
    fn normalize(&mut self, tokens: &mut Tokens) {
        tokens.iter_mut().for_each(|token| {
            if let Some(replacement) = self.pairs.read().unwrap().get(token.inner_ref()) {
                *token = Token::from(replacement.to_string());
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        normalizer::{replace::TokenReplacer, TextNormalizer},
        tokens,
    };

    #[test]
    fn test_normalizer_replace_token() {
        let mut tokens =
            tokens!["The", "lazy", "brown", "dog", "jumps", "over", "the", "quick", "fox"];

        let mut pairs = HashMap::new();
        pairs.insert("lazy".into(), "quick");
        pairs.insert("quick".into(), "lazy");
        pairs.insert("dog".into(), "fox");
        pairs.insert("fox".into(), "dog");

        let mut normalizer = TokenReplacer::new(pairs);
        normalizer.normalize(&mut tokens);

        assert_eq![
            tokens,
            tokens!["The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"]
        ]
    }
}
