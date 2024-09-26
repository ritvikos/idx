pub mod case;
pub mod punctuation;
pub mod replace;
pub mod stopwords;

pub use stopwords::Stopwords;

use crate::token::Tokens;

pub trait TextNormalizerClone {
    fn clone_box(&self) -> Box<dyn TextNormalizer>;
}

impl<T> TextNormalizerClone for T
where
    T: 'static + TextNormalizer + Clone,
{
    fn clone_box(&self) -> Box<dyn TextNormalizer> {
        Box::new(self.clone())
    }
}

pub trait TextNormalizer: TextNormalizerClone + std::fmt::Debug + Send + Sync {
    fn normalize(&mut self, tokens: &mut Tokens);
}

impl Clone for Box<dyn TextNormalizer> {
    fn clone(&self) -> Box<dyn TextNormalizer> {
        self.clone_box()
    }
}

#[derive(Clone, Debug)]
pub struct Normalizer(Box<dyn TextNormalizer>);

impl Normalizer {
    pub fn new<T: TextNormalizer + 'static>(normalizer: T) -> Self {
        Self(Box::new(normalizer))
    }
}

impl Normalizer {
    pub fn normalize(&mut self, tokens: &mut Tokens) {
        self.0.normalize(tokens);
    }
}

#[derive(Clone, Debug)]
pub struct NormalizerPipeline(Vec<Box<dyn TextNormalizer>>);

impl NormalizerPipeline {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert(&mut self, normalizer: Box<dyn TextNormalizer>) -> &mut Self {
        self.0.push(normalizer);
        self
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn run(&mut self, tokens: &mut Tokens) {
        self.0.iter_mut().for_each(|normalizer| {
            normalizer.normalize(tokens);
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        normalizer::{case::Lowercase, punctuation::Punctuation, NormalizerPipeline, Stopwords},
        tokenizer::{TextTokenizer, Whitespace},
        tokens,
    };

    #[test]
    pub fn test_pipeline() {
        let stopwords = [
            "is", "and", "with", "or", "it", "can", "on", "of", "the", "has", "a", "an", "you",
            "to", "at",
        ]
        .iter()
        .map(|&word| word.to_string())
        .collect::<Vec<_>>();

        let mut document = r#"
            Rust is blazingly fast and memory-efficient: with no runtime or garbage collector, it can power performance-critical services, run on embedded devices, and easily integrate with other languages.
            Rust’s rich type system and ownership model guarantee memory-safety and thread-safety — enabling you to eliminate many classes of bugs at compile-time.
            Rust has great documentation, a friendly compiler with useful error messages, and top-notch tooling — an integrated package manager and build tool, smart multi-editor support with auto-completion and type inspections, an auto-formatter, and more.
        "#;

        // TODO: Replace with standard tokenizer.
        let mut tokenizer = Whitespace::new();
        let mut tokens = tokenizer.tokenize(&mut document);
        let mut pipeline = NormalizerPipeline::new();

        let punctuation_normalizer = Punctuation::new();
        let case_normalizer = Lowercase::new();
        let stopwords_normalizer = Stopwords::new(stopwords);

        pipeline.insert(Box::new(punctuation_normalizer));
        pipeline.insert(Box::new(case_normalizer));
        pipeline.insert(Box::new(stopwords_normalizer));
        pipeline.run(&mut tokens);

        // println!("tokens: {tokens:?}");
    }

    #[test]
    pub fn test_normalizer_pipeline() {
        const STOPWORDS_ALL: [&str; 3] = ["the", "and", "in"];
        let mut tokens = tokens!["the", "cat", "in", "the", "hat", "and", "bat"];

        let mut pipeline = NormalizerPipeline::new();
        let stopwords_normalizer = Stopwords::new(STOPWORDS_ALL);
        let case_normalizer = Lowercase::new();

        pipeline.insert(Box::new(case_normalizer));
        pipeline.insert(Box::new(stopwords_normalizer));

        pipeline.run(&mut tokens);

        assert_eq!(tokens, tokens!["cat", "hat", "bat"]);
    }

    #[test]
    fn test_normalizer_with_punctuation() {
        const STOPWORDS_PUNCTUATION: [&str; 3] = ["the", "and", "in"];
        let mut tokens = tokens!["the", "cat.", "in", "the!", "hat", "and?", "bat,",];

        let mut pipeline = NormalizerPipeline::new();
        let stopwords_normalizer = Stopwords::new(STOPWORDS_PUNCTUATION);
        let punctuation_normalizer = Punctuation::new();

        pipeline.insert(Box::new(punctuation_normalizer));
        pipeline.insert(Box::new(stopwords_normalizer));

        pipeline.run(&mut tokens);

        assert_eq!(tokens, tokens!["cat", "hat", "bat"]);
    }
}
