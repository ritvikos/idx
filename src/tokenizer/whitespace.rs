use crate::tokenizer::TextTokenizer;

#[derive(Debug, Default)]
pub struct Whitespace;

impl Whitespace {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TextTokenizer for Whitespace {
    fn tokenize(&mut self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|token| token.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::{TextTokenizer, Whitespace};

    #[test]
    fn test_whitespace() {
        let mut tokenizer = Whitespace::new();
        let text = "This is a test";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["This", "is", "a", "test"]);
    }

    #[test]
    fn test_whitespace_empty_string() {
        let mut tokenizer = Whitespace::new();
        let text = "";
        let tokens = tokenizer.tokenize(text);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_whitespace_multiple() {
        let mut tokenizer = Whitespace::new();
        let text = "This  is    a test";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["This", "is", "a", "test"]);
    }

    #[test]
    fn test_whitespace_leading_and_trailing() {
        let mut tokenizer = Whitespace::new();
        let text = "   This is a test   ";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["This", "is", "a", "test"]);
    }

    #[test]
    fn test_whitespace_non_ascii() {
        let mut tokenizer = Whitespace::new();
        let text = "This is\u{00A0}a\u{3000}test";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["This", "is", "a", "test"]);
    }

    #[test]
    fn test_whitespace_string() {
        let mut tokenizer = Whitespace::new();
        let text = "   \t\n  ";
        let tokens = tokenizer.tokenize(text);
        assert!(tokens.is_empty());
    }
}
