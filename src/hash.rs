extern crate xxhash_rust;

use std::{
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
};

use xxhash_rust::xxh3::Xxh3;

pub trait CustomHash: Hasher + Default {
    fn new() -> Self;
    fn hash_key<K: Hash + ?Sized>(&mut self, key: &K) -> u64;
    fn reset(&mut self);
}

#[derive(Debug, Default)]
pub struct CustomHasher<T: CustomHash = DefaultHasher>(pub T);

impl<T: CustomHash> CustomHasher<T> {
    pub fn new() -> Self {
        Self(T::new())
    }

    pub fn finalize<K: Hash + ?Sized>(&mut self, key: &K) -> u64 {
        self.0.hash_key(key)
    }

    pub fn reset(&mut self) {
        self.0.reset();
    }
}

#[derive(Default)]
pub struct DefaultHasher {
    hasher: Xxh3,
}

impl CustomHash for DefaultHasher {
    fn new() -> Self {
        Self {
            hasher: Xxh3::new(),
        }
    }

    fn hash_key<K: Hash + ?Sized>(&mut self, key: &K) -> u64 {
        key.hash(self);
        self.finish()
    }

    fn reset(&mut self) {
        self.hasher.reset();
    }
}

impl Hasher for DefaultHasher {
    fn finish(&self) -> u64 {
        self.hasher.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.hasher.write(bytes);
    }
}

impl Debug for DefaultHasher {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultHasher")
            .field("hasher", &"<Hasher>")
            .finish()
    }
}
