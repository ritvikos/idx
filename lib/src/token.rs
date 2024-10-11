use std::{
    ops::{Deref, DerefMut},
    slice::{Iter, IterMut},
};

// TODO: Use generics to perform ops on
// byte arrays and support multiple types.
#[derive(Debug, Default, Hash, Eq, PartialEq)]
pub struct Token(String);

impl Token {
    pub fn inner(self) -> String {
        self.0
    }

    pub fn inner_ref(&self) -> &String {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut String {
        &mut self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for Token {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Token> for String {
    fn from(value: Token) -> Self {
        value.0
    }
}

impl DerefMut for Token {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<String> for Token {
    fn from(value: String) -> Self {
        Token(value)
    }
}

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        Token(String::from(value))
    }
}

impl From<&&str> for Token {
    fn from(value: &&str) -> Self {
        Token(String::from(*value))
    }
}

impl AsMut<str> for Token {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub type Tokens = TokenIter<Token>;
pub type TokenIter<T> = TokenVec<T>;

#[derive(Debug)]
pub struct TokenVec<T: Into<Token> + PartialEq>(Vec<T>);

impl<T: Into<Token> + PartialEq> TokenVec<T> {
    #[inline]
    pub fn push(&mut self, item: T) {
        self.0.push(item)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn term_count(&self, term: &T) -> usize {
        self.iter().filter(|&element| element == term).count()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.0.iter()
    }

    pub fn for_each_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        self.iter_mut().for_each(|item| {
            f(item);
        })
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.0.iter_mut()
    }

    #[inline]
    pub fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        self.0.retain_mut(f)
    }
}

impl<T: Into<Token> + PartialEq> From<Vec<T>> for TokenVec<T> {
    fn from(value: Vec<T>) -> Self {
        TokenVec(value)
    }
}

impl<T: Into<Token> + PartialEq> PartialEq for TokenVec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Into<Token> + PartialEq> FromIterator<T> for TokenVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        TokenVec(iter.into_iter().collect())
    }
}

impl<T: Into<Token> + PartialEq> IntoIterator for TokenVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[macro_export]
macro_rules! tokens {
    ( $( $token:expr ),* $(,)? ) => {{
        $crate::token::TokenVec::from(vec![
            $( $crate::token::Token::from($token) ),*
        ])
    }};
}
