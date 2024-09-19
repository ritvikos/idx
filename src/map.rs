use std::{collections::HashMap, hash::Hash};

use crate::util::Counter;

#[derive(Debug, Default)]
pub struct TermCounter<K>
where
    K: Eq + Hash + Into<String>,
{
    inner: HashMap<K, Counter<usize>>,
}

impl<K> TermCounter<K>
where
    K: Eq + Hash + Into<String>,
{
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn insert(&mut self, key: K) {
        self.inner
            .entry(key)
            .and_modify(Counter::increment)
            .or_insert_with(|| Counter::new(1));
    }

    pub fn get<Q>(&self, key: Q) -> Option<&Counter<usize>>
    where
        Q: Into<K>,
    {
        self.inner.get(&key.into())
    }

    pub fn reset(&mut self) {
        self.inner.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::{map::TermCounter, tokens, util::Counter};

    #[test]
    fn test_map_frequency_basic() {
        let mut map = TermCounter::new();

        let tokens = tokens!["apple", "banana", "apple", "orange", "banana", "apple"];

        for token in tokens {
            map.insert(token);
        }

        assert_eq!(map.get("apple"), Some(&Counter::new(3)));
        assert_eq!(map.get("banana"), Some(&Counter::new(2)));
        assert_eq!(map.get("orange"), Some(&Counter::new(1)));
        assert_eq!(map.get("pineapple"), None);
    }
}
