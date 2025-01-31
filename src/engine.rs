use std::fmt::Debug;

use crate::descriptor::Descriptor;

use idx::{
    aggregate::{Aggregator, HashAggregator, Order},
    index::Indexer,
    normalizer::NormalizerPipeline,
    query::Query,
    score::{Score, Scorer, TfIdfScorer},
    tokenizer::Tokenizer,
};

#[derive(Debug)]
pub struct IdxFacade<I: Indexer> {
    pub index: I,
    pub tokenizer: Tokenizer,
    pub pipeline: NormalizerPipeline,
}

impl<I: Indexer> IdxFacade<I> {
    pub fn new(
        capacity: usize,
        threshold: usize,
        tokenizer: Tokenizer,
        pipeline: NormalizerPipeline,
    ) -> Self {
        Self {
            index: Indexer::new(capacity, threshold),
            tokenizer,
            pipeline,
        }
    }

    pub fn insert(&mut self, descriptor: Descriptor<<I as Indexer>::R>) {
        let mut tokens = descriptor.tokenize(&mut self.tokenizer);
        let resource = descriptor.resource();
        let word_count = tokens.count();

        if !self.pipeline.is_empty() {
            self.pipeline.run(&mut tokens);
        }

        self.index.insert(resource, word_count, &mut tokens);
    }

    pub fn get(&self, query: Query) -> Vec<I::R> {
        let reader = self.index.reader();

        let mut tokenizer = self.tokenizer.clone();
        let mut pipeline = self.pipeline.clone();

        let mut tokens = query.tokenize(&mut tokenizer);

        if !self.pipeline.is_empty() {
            pipeline.run(&mut tokens);
        }

        let hash_aggregator = HashAggregator::new();
        let mut aggregator = Aggregator::new(hash_aggregator);

        let tfidf_scorer = TfIdfScorer::new(&reader);
        let mut scorer = Scorer::new(tfidf_scorer);

        // Vec<(index, score)>
        scorer.score_and_apply(
            |scores| {
                for score in scores {
                    aggregator.insert(score.0, score.1);
                }
            },
            tokens,
        );

        // temporary
        // FIXME: create unified interface for aggregator
        let sorted = aggregator.sort_by(Order::Descending);

        sorted
            .iter()
            .filter_map(|(index, _)| self.index.get(*index))
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use idx::{
        index::Index,
        normalizer::{case::Lowercase, punctuation::Punctuation, NormalizerPipeline, Stopwords},
        tokenizer::{Standard, Tokenizer},
    };

    use crate::{
        descriptor::Descriptor,
        engine::{IdxFacade, Query},
    };

    fn tiny_test_corpus() -> Vec<String> {
        [
            "the cat sat on the mat",
            "the cat sat",
            "the dog barked",
            "penguin is a nice animal",
        ]
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
    }

    #[test]
    fn test_indexer_and_engine_basic() {
        let corpus = tiny_test_corpus();
        let tokenizer = Tokenizer::Standard(Standard::new());

        let mut pipeline = NormalizerPipeline::new();
        pipeline.insert(Box::new(Lowercase::new()));
        pipeline.insert(Box::new(Punctuation::new()));
        pipeline.insert(Box::new(
            Stopwords::load("assets/stopwords/en.txt").unwrap(),
        ));

        let mut engine: IdxFacade<Index<String>> =
            IdxFacade::new(10, 30, tokenizer.clone(), pipeline);

        for document in corpus {
            let descriptor = Descriptor::new(document.clone(), document.into());
            engine.insert(descriptor);
        }

        let target = "cat sat";

        let query = Query::new(target);
        let collection = engine.get(query);
        println!("collection: {collection:?}");
    }

    #[derive(Clone, Debug)]
    struct Webpage {
        url: String,
        title: String,
        excerpt: String,
    }

    fn tiny_test_webpage_corpus() -> Vec<Webpage> {
        vec![
        Webpage {
            url: String::from("https://example.com/rust-guide"),
            title: String::from("Rust Programming Guide"),
            excerpt: String::from("A comprehensive guide to Rust programming, covering the basics to advanced topics."),
        },
        Webpage {
            url: String::from("https://example.com/webdev-trends"),
            title: String::from("Top Web Development Trends in 2024"),
            excerpt: String::from("Explore the latest trends in web development, from frameworks to tools."),
        },
        Webpage {
            url: String::from("https://example.com/ai-future"),
            title: String::from("The Future of AI"),
            excerpt: String::from("A look into how AI is shaping the future across various industries."),
        },
        Webpage {
            url: String::from("https://example.com/cybersecurity-basics"),
            title: String::from("Cybersecurity Essentials"),
            excerpt: String::from("Learn the fundamentals of cybersecurity and how to protect digital assets."),
        },
    ]
    }

    #[test]
    fn test_indexer_and_engine_with_structure() {
        let corpus = tiny_test_webpage_corpus();
        let tokenizer = Tokenizer::Standard(Standard::new());

        let mut pipeline = NormalizerPipeline::new();
        pipeline.insert(Box::new(Lowercase::new()));
        pipeline.insert(Box::new(Punctuation::new()));
        pipeline.insert(Box::new(
            Stopwords::load("assets/stopwords/en.txt").unwrap(),
        ));

        let mut engine: IdxFacade<Index<Webpage>> =
            IdxFacade::new(10, 30, tokenizer.clone(), pipeline);

        for document in corpus {
            let doc = document.excerpt.clone();
            let descriptor = Descriptor::new(document.clone(), doc.into());
            engine.insert(descriptor);
        }

        let target = "AI";

        let query = Query::new(target);
        let collection = engine.get(query);
        println!("collection: {collection:?}");
    }
}
