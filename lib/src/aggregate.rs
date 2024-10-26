use std::{fmt::Debug, marker::PhantomData};

use hashbrown::{hash_map::Iter, HashMap};
use num_traits::{Float, Unsigned};

pub trait Aggregate<K: Unsigned, V: Float>: Debug {
    type Iter<'a>: Iterator<Item = (&'a K, &'a V)>
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    fn insert(&mut self, key: K, value: V);
    fn iter(&self) -> Self::Iter<'_>;
}

#[derive(Debug)]
pub struct Aggregator<A: Aggregate<K, V>, K: Unsigned, V: Float> {
    inner: A,
    _marker: PhantomData<(K, V)>,
}

impl<A: Aggregate<K, V>, K: Unsigned, V: Float> Aggregator<A, K, V> {
    pub fn new(strategy: A) -> Self {
        Self {
            inner: strategy,
            _marker: PhantomData,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.inner.insert(key, value);
    }

    pub fn iter(&self) -> <A as Aggregate<K, V>>::Iter<'_> {
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

impl Aggregate<usize, f32> for HashAggregator {
    type Iter<'a> = Iter<'a, usize, f32>;

    fn insert(&mut self, key: usize, value: f32) {
        self.inner
            .entry(key)
            .and_modify(|existing| *existing += value)
            .or_insert(value);
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.inner.iter()
    }
}
