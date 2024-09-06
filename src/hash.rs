extern crate xxhash_rust;

use std::{
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
};

use xxhash_rust::xxh3::Xxh3;

pub trait CustomHash: Hasher + Default {
    /// Create a new instance.
    fn new() -> Self;

    /// Perform hashing on the inner field.
    fn hash_inner<K: Hash>(&mut self, key: K) -> u64;
}

#[derive(Debug)]
pub struct CustomHasher<T: CustomHash>(pub T);

impl<T: CustomHash> CustomHasher<T> {
    pub fn new(hasher: T) -> Self {
        Self(hasher)
    }

    pub fn finalize(&mut self, key: &String) -> u64 {
        self.0.hash_inner(key)
    }
}

#[derive(Default)]
pub struct DefaultHasher {
    inner: u64,
    hasher: Xxh3,
}

impl CustomHash for DefaultHasher {
    fn new() -> Self {
        Self {
            inner: 0,
            hasher: Xxh3::new(),
        }
    }

    fn hash_inner<K: Hash>(&mut self, key: K) -> u64 {
        key.hash(&mut self.hasher);
        self.inner = self.hasher.finish();
        self.inner
    }
}

impl Hasher for DefaultHasher {
    fn finish(&self) -> u64 {
        self.inner
    }

    fn write(&mut self, bytes: &[u8]) {
        self.hasher.write(bytes);
    }
}

impl Debug for DefaultHasher {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultHasher")
            .field("inner", &self.inner)
            .field("hasher", &"<Hasher>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::hash::{CustomHasher, DefaultHasher};

    #[test]
    fn test_hasher_basic() {
        let default = DefaultHasher::default();
        let mut hasher = CustomHasher::new(default);

        let key = String::from("sample");
        let hash = hasher.finalize(&key);
        println!("hash: {hash}");
    }
}
