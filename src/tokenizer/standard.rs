use std::{collections::HashSet, sync::OnceLock};

use crate::tokenizer::TextTokenizer;

fn delimiters() -> &'static HashSet<&'static char> {
    static SET: OnceLock<HashSet<&char>> = OnceLock::new();
    SET.get_or_init(|| {
        [
            ' ', ',', ';', '!', '@', '#', '$', '%', '^', '.', '-', '(', ')', '{', '}', '[', ']',
            '\'', '\'', '\"', '\"', '<', '>', '\t', '\r', '\n',
        ]
        .iter()
        .collect::<HashSet<&char>>()
    })
}

#[derive(Debug, Default)]
pub struct Standard {}

impl Standard {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TextTokenizer for Standard {
    fn tokenize(&mut self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut token = String::new();

        for ch in text.chars() {
            if !delimiters().contains(&ch) {
                token.push(ch);
            } else {
                if !token.is_empty() {
                    tokens.push(std::mem::take(&mut token));
                }
            }
        }

        if !token.is_empty() {
            tokens.push(token);
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::{Standard, TextTokenizer};

    #[test]
    fn test_standard_basic() {
        let text = "The quick brown fox jumps over the lazy dog";

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(text);

        assert_eq!(
            tokens,
            vec!["The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"]
        );
    }

    #[test]
    fn test_standard_with_punctuation() {
        let text = String::from("Hello, world! This is a test.");

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(&text);

        assert_eq!(tokens, vec!["Hello", "world", "This", "is", "a", "test"]);
    }

    #[test]
    fn test_standard_empty_string() {
        let text = String::new();

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(&text);

        assert_eq!(tokens, vec![] as Vec<&str>);
    }

    #[test]
    fn test_standard_multiple_spaces() {
        let text = String::from("The  quick   brown fox");

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(&text);

        assert_eq!(tokens, vec!["The", "quick", "brown", "fox"]);
    }

    #[test]
    fn test_tokenizer_with_newlines() {
        let text = String::from("The quick\nbrown\nfox");

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(&text);

        assert_eq!(tokens, vec!["The", "quick", "brown", "fox"]);
    }

    #[test]
    fn test_standard_with_tabs() {
        let text = String::from("The\tquick\tbrown\tfox");

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(&text);

        assert_eq!(tokens, vec!["The", "quick", "brown", "fox"]);
    }

    #[test]
    fn test_standard_unicode() {
        let text = String::from("एकाधिक - ಭಾಷೆಗಳು - work");

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(&text);

        assert_eq!(tokens, vec!["एकाधिक", "ಭಾಷೆಗಳು", "work"]);
    }

    #[test]
    fn test_standard_mixed_whitespace() {
        let text = String::from("The quick\tbrown\nfox  jumps\tover\nthe lazy\tdog");

        let mut tokenizer = Standard::new();
        let tokens = tokenizer.tokenize(&text);

        assert_eq!(
            tokens,
            vec!["The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog",]
        );
    }
}
