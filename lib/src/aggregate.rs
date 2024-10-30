use std::fmt::Debug;

use hashbrown::hash_map::{HashMap, Iter};

// TODO:
// - add limit
pub trait Aggregation: Debug {
    type Key;
    type Value;

    type Iter<'a>: Iterator<Item = (&'a Self::Key, &'a Self::Value)>
    where
        Self: 'a;

    fn insert(&mut self, key: Self::Key, value: Self::Value);
    fn iter(&self) -> Self::Iter<'_>;

    // FIXME: create unified interface for aggregator
    fn sort_by(&self, order: Order) -> Vec<(Self::Key, Self::Value)>;
}

#[derive(Debug)]
pub enum Order {
    Ascending,
    Descending,
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

    pub fn sort_by(&self, order: Order) -> Vec<(A::Key, A::Value)> {
        self.inner.sort_by(order)
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

    fn sort_by(&self, order: Order) -> Vec<(Self::Key, Self::Value)> {
        let mut items: Vec<(Self::Key, Self::Value)> =
            self.inner.iter().map(|(&k, &v)| (k, v)).collect();

        match order {
            Order::Ascending => items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap()),
            Order::Descending => items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()),
        }

        items
    }
}
