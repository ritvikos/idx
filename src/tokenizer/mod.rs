mod standard;
mod whitespace;

pub use {standard::Standard, whitespace::Whitespace};

pub type Tokens = Vec<Token>;

#[derive(Debug, PartialEq, Eq)]
pub struct Token(Vec<u8>);

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        Token(Vec::from(value))
    }
}

impl From<&&str> for Token {
    fn from(value: &&str) -> Self {
        Token(Vec::from(*value))
    }
}

impl From<&[u8]> for Token {
    fn from(value: &[u8]) -> Self {
        Token(Vec::from(value))
    }
}

pub trait TextTokenizer {
    fn tokenize<T: AsRef<str>>(&mut self, text: T) -> Tokens;
}

#[macro_export]
macro_rules! tokens {
    ( $( $token:expr ),* $(,)? ) => {{
        vec![
            $( Token::from($token) ),*
        ]
    }};
}

// pub struct Tokenizer<T: TextTokenizer>(T);

// impl<'a, T: TextTokenizer<'a>> Tokenizer<T> {
// pub fn new(kind: T) -> Self {
// Self(kind)
// }

// pub fn tokenize(&mut self, text: &str) -> Vec<Token> {
// self.0.tokenize(text)
// }
// }

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
}
