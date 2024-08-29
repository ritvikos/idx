mod standard;
mod whitespace;

pub use {standard::Standard, whitespace::Whitespace};

pub trait TextTokenizer {
    fn tokenize(&mut self, text: &str) -> Vec<String>;
}

pub struct Tokenizer<T: TextTokenizer>(T);

impl<T: TextTokenizer> Tokenizer<T> {
    pub fn new(kind: T) -> Self {
        Self(kind)
    }

    pub fn tokenize(&mut self, text: &str) -> Vec<String> {
        self.0.tokenize(text)
    }
}

#[cfg(test)]
mod tests {
    use super::{Tokenizer, Whitespace};

    #[test]
    fn test_tokenizer_whitespace() {
        let text = "The quick brown fox jumps over the lazy dog";
        let expected = vec![
            "The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog",
        ];

        let mut tokenizer = Tokenizer::new(Whitespace);
        let tokens = tokenizer.tokenize(text);

        assert_eq!(tokens, expected);
    }
}
