extern crate hashbrown;

use std::{cell::RefCell, hash::Hash, num::NonZeroUsize};

use hashbrown::{
    hash_map::HashMap,
    hash_set::{HashSet, Iter},
};

use crate::util::Counter;

#[derive(Clone, Copy, Debug)]
pub struct TfIdf {
    /// File index
    index: usize,

    /// Tf-Idf score
    score: f32,
}

impl TfIdf {
    #[inline]
    pub fn new(index: usize, score: f32) -> Self {
        Self { index, score }
    }

    #[inline]
    pub fn get_index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn get_score(&self) -> f32 {
        self.score
    }
}

// TODO: Ensure that same files are not added more than once, maybe use another data structure.
#[derive(Debug)]
pub struct FileIndex {
    inner: Vec<FileEntry>,
}

impl FileIndex {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn insert(&mut self, value: FileEntry) -> usize {
        self.inner.push(value);
        self.len() - 1
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&FileEntry> {
        self.inner.get(index)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FileEntry {
    path: String,

    // Word count
    count: NonZeroUsize,
}

impl FileEntry {
    pub fn new<S: Into<String>>(path: S, word_count: usize) -> Self {
        Self {
            path: path.into(),

            // SAFETY:
            // - The value must not be zero, so empty documents are not indexed.
            count: unsafe { NonZeroUsize::new_unchecked(word_count) },
        }
    }

    pub fn count(&self) -> usize {
        self.count.get()
    }
}

// TODO: Handle threshold.
#[derive(Debug)]
pub struct InvertedIndex {
    inner: HashMap<String, IdfEntry>,
}

impl InvertedIndex {
    #[inline]
    pub fn new(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    /// Adds a term to the inverted index with its associated `RefEntry`.    
    #[inline]
    pub fn add_term(&mut self, term: String, tf_entry: TfEntry) {
        // TODO: Track default capacity and threshold.
        self.inner
            .entry_ref(&term)
            .and_modify(|entry| entry.add_entry(RefEntry::new(tf_entry)))
            .or_insert_with(|| {
                let mut set = HashSet::new();
                set.insert(RefEntry::new(tf_entry));
                IdfEntry { entries: set }
            });
    }

    /// Returns an immutable reference to the `IdfEntry` for a given term.
    #[inline]
    pub fn get_term_entries(&self, term: &str) -> Option<&IdfEntry> {
        match self.inner.get(term) {
            Some(entry) => Some(entry),
            None => None,
        }
    }

    /// Retrieves the `IdfEntry` associated with the specified `term` and applies
    /// the function `f` to it, returning the result wrapped in `Option`.
    /// If the term is not found, it returns `None`.
    #[inline]
    pub fn get_entry_with<O>(&self, term: &str, f: impl FnOnce(&IdfEntry) -> O) -> Option<O> {
        self.get_term_entries(term).map(f)
    }
}

pub struct EntriesIter<'a> {
    inner: Iter<'a, RefEntry>,
}

impl<'a> Iterator for EntriesIter<'a> {
    type Item = &'a RefEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

// SCANNS
// IDFIndexEntry keeps track of the token associated with the entry,
// the number of files that contain the term,
#[derive(Debug, Default)]
pub struct IdfEntry {
    // TODO:
    // - Maybe use better data structure for this use-case.
    // - Track `threshold` for `IdfEntry` somewhere else.
    entries: HashSet<RefEntry>,
}

impl IdfEntry {
    #[inline]
    pub fn with_capacity(capacity: usize, threshold: usize) -> Self {
        Self {
            entries: HashSet::with_capacity(capacity),
        }
    }

    /// Number of documents containing the term.
    #[inline]
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Adds a `RefEntry` to this `IdfEntry`.
    pub fn add_entry(&mut self, entry: RefEntry) {
        if let Some(entry) = self.entries.get(&entry) {
            entry.increment_frequency();
            return;
        }

        self.entries.insert(entry);
    }

    /// Retrieves all the entries associated with this `IdfEntry`.
    #[inline]
    pub fn get_entries(&self) -> &HashSet<RefEntry> {
        &self.entries
    }

    #[inline]
    pub fn iter(&self) -> EntriesIter {
        EntriesIter {
            inner: self.entries.iter(),
        }
    }

    // #[inline]
    // pub fn should_flush(&self) -> bool {
    //     self.range() > self.threshold
    // }

    // #[inline]
    // fn range(&self) -> usize {
    //     (self.entries.len() * 100) / self.entries.capacity()
}

#[derive(Clone, Debug, Eq)]
pub struct RefEntry(RefCell<TfEntry>);

impl RefEntry {
    #[inline]
    pub fn new(entry: TfEntry) -> Self {
        Self(RefCell::new(entry))
    }

    #[inline]
    pub fn tf_entry(&self) -> TfEntry {
        *self.0.borrow()
    }

    #[inline]
    pub fn get_index(&self) -> usize {
        self.tf_entry().get_index()
    }

    #[inline]
    pub fn get_frequency(&self) -> Counter<usize> {
        self.tf_entry().get_frequency()
    }

    #[inline]
    pub fn increment_frequency(&self) {
        self.0.borrow_mut().increment_frequency()
    }
}

impl Hash for RefEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.borrow().hash(state);
    }
}

impl PartialEq for RefEntry {
    fn eq(&self, other: &Self) -> bool {
        self.0.borrow().index == other.0.borrow().index
    }
}

// SCANNS
// TFIndexEntry stores the index associated with
// a file, the frequency of a term in that file
#[derive(Clone, Copy, Debug, Eq)]
pub struct TfEntry {
    /// Index associated with file.
    index: usize,

    /// Frequency of term in the file.
    frequency: Counter<usize>,
}

impl TfEntry {
    #[inline]
    pub fn get_frequency(&mut self) -> Counter<usize> {
        self.frequency
    }

    #[inline]
    pub fn get_index(&mut self) -> usize {
        self.index
    }

    #[inline]
    pub fn increment_frequency(&mut self) {
        self.frequency.increment();
    }
}

impl TfEntry {
    pub fn new(index: usize, frequency: usize) -> Self {
        Self {
            index,
            frequency: Counter::new(frequency),
        }
    }
}

impl Hash for TfEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}

impl PartialEq for TfEntry {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

#[derive(Debug)]
pub struct Field {
    inner: Vec<TfIdf>,
}

impl From<Vec<TfIdf>> for Field {
    fn from(value: Vec<TfIdf>) -> Self {
        Self { inner: value }
    }
}

#[derive(Debug)]
pub struct Collection {
    inner: Vec<Vec<TfIdf>>,
}

impl Collection {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, field: Vec<TfIdf>) {
        self.inner.push(field)
    }
}

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
    use crate::{core::TermCounter, tokens, util::Counter};

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
