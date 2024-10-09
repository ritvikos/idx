use crate::token::Tokens;

pub trait TextFilter {
    fn filter(&mut self, tokens: &mut Tokens);
}

pub struct Filter<T: TextFilter>(T);

impl<T: TextFilter> Filter<T> {
    pub fn new(kind: T) -> Self {
        Self(kind)
    }

    pub fn filter(&mut self, tokens: &mut Tokens) {
        self.0.filter(tokens);
    }
}

pub struct FilterPipeline<T: TextFilter>(Vec<T>);

impl<T: TextFilter> Default for FilterPipeline<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TextFilter> FilterPipeline<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert(&mut self, filter: T) -> &mut Self {
        self.0.push(filter);
        self
    }
}

// #[cfg(test)]
// mod tests {
// use super::{Filter, FilterPipeline};

// #[test]
// fn test_filter_pipeline() {
// let mut pipeline = FilterPipeline::new();
// }
// }
