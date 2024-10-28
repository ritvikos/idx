use std::fmt::Debug;

use hashbrown::hash_map::{HashMap, Iter};

pub trait Aggregation: Debug {
    type Key;
    type Value;

    type Iter<'a>: Iterator<Item = (&'a Self::Key, &'a Self::Value)>
    where
        Self: 'a;

    fn insert(&mut self, key: Self::Key, value: Self::Value);
    fn iter(&self) -> Self::Iter<'_>;
}

#[derive(Debug)]
pub struct Aggregator<A: Aggregation> {
    inner: A,
}

impl<A: Aggregation> Aggregator<A> {
    pub fn new(strategy: A) -> Self {
        Self { inner: strategy }
    }

    pub fn insert(&mut self, key: A::Key, value: A::Value) {
        self.inner.insert(key, value);
    }

    pub fn iter(&self) -> A::Iter<'_> {
        self.inner.iter()
    }
}

#[derive(Debug)]
pub struct HashAggregator {
    inner: HashMap<usize, f32>,
}

impl HashAggregator {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }
}

impl Aggregation for HashAggregator {
    type Key = usize;
    type Value = f32;
    type Iter<'a> = Iter<'a, Self::Key, Self::Value>;

    fn insert(&mut self, key: Self::Key, value: Self::Value) {
        self.inner
            .entry(key)
            .and_modify(|existing| *existing += value)
            .or_insert(value);
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.inner.iter()
    }
}
