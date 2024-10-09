use crate::{normalizer::TextNormalizer, token::Tokens};

#[derive(Clone, Debug, Default)]
pub struct Lowercase;

impl Lowercase {
    pub fn new() -> Self {
        Self
    }
}

impl TextNormalizer for Lowercase {
    fn normalize(&mut self, tokens: &mut Tokens) {
        tokens.iter_mut().for_each(|token| {
            token.as_mut().make_ascii_lowercase();
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Uppercase;

impl Uppercase {
    pub fn new() -> Self {
        Self
    }
}

impl TextNormalizer for Uppercase {
    fn normalize(&mut self, tokens: &mut Tokens) {
        tokens.iter_mut().for_each(|token| {
            token.as_mut().make_ascii_uppercase();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Lowercase;
    use crate::{
        normalizer::{case::Uppercase, TextNormalizer},
        tokens,
    };

    #[test]
    fn test_normalizer_lowercase() {
        let mut tokens = tokens!["The", "TokeniZED", "String"];
        let mut normalizer = Lowercase::new();
        normalizer.normalize(&mut tokens);
        assert_eq!(tokens, tokens!["the", "tokenized", "string"])
    }

    #[test]
    fn test_normalizer_uppercase() {
        let mut tokens = tokens!["the", "TokeniZED", "STRING"];
        let mut normalizer = Uppercase::new();
        normalizer.normalize(&mut tokens);
        assert_eq!(tokens, tokens!["THE", "TOKENIZED", "STRING"])
    }

    #[test]
    fn test_normalizer_all_lowercase() {
        let mut tokens = tokens!["the", "tokenized", "string"];
        let mut normalizer = Lowercase::new();
        normalizer.normalize(&mut tokens);
        assert_eq!(tokens, tokens!["the", "tokenized", "string"])
    }

    #[test]
    fn test_normalizer_all_uppercase() {
        let mut tokens = tokens!["THE", "TOKENIZED", "STRING"];
        let mut normalizer = Uppercase::new();
        normalizer.normalize(&mut tokens);
        assert_eq!(tokens, tokens!["THE", "TOKENIZED", "STRING"])
    }
}
