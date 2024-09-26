mod html;
mod standard;
mod whitespace;

pub use {standard::Standard, whitespace::Whitespace};

use crate::token::{Token, Tokens};

#[derive(Clone, Debug)]
pub enum Tokenizer {
    Standard(Standard),
    Whitespace(Whitespace),
}

impl Tokenizer {
    pub fn tokenize(&mut self, text: &str) -> Tokens {
        match self {
            Tokenizer::Standard(tokenizer) => tokenizer.tokenize(text),
            Tokenizer::Whitespace(tokenizer) => tokenizer.tokenize(text),
        }
    }
}

pub trait TextTokenizer {
    fn tokenize<T: AsRef<str>>(&mut self, text: T) -> Tokens;
}

// TODO
// 1. HTML Tokenizer
// 2. Regex Tokenizer

#[cfg(test)]
mod tests {
    // use super::{Tokenizer, Whitespace};

    //     #[test]
    //     fn test_tokenizer_whitespace() {
    //         let text = "The quick brown fox jumps over the lazy dog";
    //         let expected = vec![
    //             "The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog",
    //         ];

    //         let mut tokenizer = Tokenizer::new(Whitespace);
    //         let tokens = tokenizer.tokenize(text);

    //         assert_eq!(tokens, expected);
    //     }

    #[test]
    fn test_tokens() {
        // let tokens = tokens!["one"];
    }
}
