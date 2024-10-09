extern crate hashbrown;

use std::fmt::Debug;

use hashbrown::hash_map::HashMap;

use crate::util::Counter;

#[derive(Debug, Default)]
// pub struct TermCounter<K>
// where
// K: Debug + Eq + Hash + ToString,
pub struct TermCounter {
    inner: HashMap<String, Counter<usize>>,
}

// impl<K> TermCounter<K>
// where
// K: Debug + Eq + Hash + ToString,
impl TermCounter {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    // pub fn insert(&mut self, key: K)
    // K: Borrow<Q> + std::cmp::Eq + Hash,
    pub fn insert(&mut self, key: String) {
        // self.inner
        //     .raw_entry_mut(&key)
        //     .and_modify(Counter::increment)
        //     .or_insert_with(|| Counter::new(1));

        self.inner
            .raw_entry_mut()
            .from_key(&key)
            .and_modify(|_, counter| counter.increment())
            .or_insert_with(|| (key, Counter::new(1)));
    }

    // pub fn get<Q>(&self, key: &Q) -> Option<&Counter<usize>>
    // where
    //     K: Borrow<Q>,
    //     Q: Hash + ?Sized + Equivalent<K> + std::cmp::Eq,
    pub fn get(&self, key: &str) -> Option<&Counter<usize>> {
        self.inner.get(key)
    }

    /// SAFETY: The caller ensures that atleast one term is present in the counter.
    //
    /// # Panics
    /// If the no term exists.
    #[inline]
    pub unsafe fn get_unchecked(&self, key: &str) -> usize {
        **self.get(key).unwrap()
    }

    pub fn reset(&mut self) {
        self.inner.clear();
    }

    // pub fn get_ref<Q>(&self, key: &Q) -> Option<&usize>
    // where
    //     K: Borrow<Q>,
    //     Q: Hash + ?Sized + Equivalent<K> + std::cmp::Eq,
    // pub fn get_ref(&self, key: &str) -> Option<&usize> {
    //     self.inner
    //         .get(key)
    //         .and_then(|counter| Some(counter.inner_ref()))
    // }
}

// This will not work for this particular implementation
// since the tokenizer is not dropped.
impl Drop for TermCounter {
    fn drop(&mut self) {
        self.reset()
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
            map.insert(token.to_string());
        }

        assert_eq!(map.get("apple"), Some(&Counter::new(3)));
        assert_eq!(map.get("banana"), Some(&Counter::new(2)));
        assert_eq!(map.get("orange"), Some(&Counter::new(1)));
        assert_eq!(map.get("pineapple"), None);
    }
}
