use std::{collections::HashSet, sync::OnceLock};

use crate::tokenizer::{TextTokenizer, Token, Tokens};

fn delimiters() -> &'static HashSet<&'static char> {
    static SET: OnceLock<HashSet<&char>> = OnceLock::new();
    SET.get_or_init(|| {
        [
            ' ', ',', ';', '!', '@', '#', '$', '%', '^', '.', '-', '(', ')', '{', '}', '[', ']',
            '\'', '\"', '<', '>', '\t', '\r', '\n',
        ]
        .iter()
        .collect::<HashSet<&char>>()
    })
}

#[derive(Debug, Default)]
pub struct Standard;

impl Standard {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TextTokenizer for Standard {
    fn tokenize<T: AsRef<str>>(&mut self, text: T) -> Tokens {
        text.as_ref()
            .split(|ch: char| delimiters().contains(&ch))
            .filter(|s| !s.is_empty())
            .map(Token::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        tokenizer::{Standard, TextTokenizer, Token},
        tokens,
    };

    #[test]
    fn test_standard_basic() {
        let text = "The quick brown fox jumps over the lazy dog";

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(text);

        assert_eq!(
            tokens,
            tokens!("The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog")
        );
    }

    #[test]
    fn test_standard_with_punctuation() {
        let text = String::from("Hello, world! This is a test.");

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(&text);

        assert_eq!(tokens, tokens!["Hello", "world", "This", "is", "a", "test"]);
    }

    #[test]
    fn test_standard_empty_string() {
        let text = String::new();

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(&text);

        assert_eq!(tokens, tokens![]);
    }

    #[test]
    fn test_standard_multiple_spaces() {
        let text = String::from("The  quick   brown fox");

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(&text);

        assert_eq!(tokens, tokens!["The", "quick", "brown", "fox"]);
    }

    #[test]
    fn test_tokenizer_with_newlines() {
        let text = String::from("The quick\nbrown\nfox");

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(&text);

        assert_eq!(tokens, tokens!["The", "quick", "brown", "fox"]);
    }

    #[test]
    fn test_standard_with_tabs() {
        let text = String::from("The\tquick\tbrown\tfox");

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(&text);

        assert_eq!(tokens, tokens!["The", "quick", "brown", "fox"]);
    }

    #[test]
    fn test_standard_unicode() {
        let text = String::from("एकाधिक - ಭಾಷೆಗಳು - work");

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(&text);

        assert_eq!(tokens, tokens!["एकाधिक", "ಭಾಷೆಗಳು", "work"]);
    }

    #[test]
    fn test_standard_mixed_whitespace() {
        let text = String::from("The quick\tbrown\nfox  jumps\tover\nthe lazy\tdog");

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(&text);

        assert_eq!(
            tokens,
            tokens!["The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog",]
        );
    }
}
