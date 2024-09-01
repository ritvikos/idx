use std::collections::HashMap;

use crate::{
    normalizer::TextNormalizer,
    tokenizer::{Token, Tokens},
};

#[derive(Debug)]
pub struct TokenReplacer<'a> {
    replacements: HashMap<&'a str, &'a str>,
}

impl<'a> TokenReplacer<'a> {
    pub fn new(replacements: HashMap<&'a str, &'a str>) -> Self {
        Self { replacements }
    }
}

impl<'a> TextNormalizer for TokenReplacer<'a> {
    fn normalize(&mut self, tokens: &mut Tokens) {
        tokens.iter_mut().for_each(|token| {
            if let Some(replacement) = self.replacements.get(token.as_str()) {
                *token = Token::from(replacement);
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

        let mut replacements = HashMap::new();
        replacements.insert("lazy", "quick");
        replacements.insert("quick", "lazy");
        replacements.insert("dog", "fox");
        replacements.insert("fox", "dog");

        let mut normalizer = TokenReplacer::new(replacements);
        normalizer.normalize(&mut tokens);

        assert_eq![
            tokens,
            tokens!["The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"]
        ]
    }
}
