// Persistent Index

// file path index stored under: file_index_<thread_id> dir.
// tfidf index stored under:     tfidf_index_<thread_id> dir.

// For each flush of any index type,
// one segment data file and one of more segment metadata files are created.

// Segment metadata files contain hashtable entries for each index type.
// Example: TFIDF Index segment metadata will contain list of IFDIndexEntry elements (array) in binary format.
//
// Segment data files contain auxiliary data structure elements.
// In FilePathIndex, the auxiliary structure is actual full file path and corresponding element
// from metadata file stores an offset to position where full file path can be found in the
// data file.
// TFIDFIndex, segment data file contains a list of lists of TFIndexEntry elements of size IndexDepth

extern crate hashbrown;

use std::{borrow::Borrow, cell::RefCell, hash::Hash, num::NonZeroUsize};

use hashbrown::{
    hash_map::{Entry, HashMap},
    hash_set::HashSet,
};

use crate::{
    descriptor::Descriptor,
    map::TermCounter,
    normalizer::NormalizerPipeline,
    tokenizer::{Token, Tokenizer},
};

/*
TfIndexEntry -> TfEntry
IDFIndexEntry -> IdfEntry
*/

/*
file -> tokens -> token | inverted_index

TF:
file_index: -
frequency: -

IDF:
no of doc containing the term: -
total doc: -
*/

// use crate::util::Counter;

// During indexing, the Indexer will perform
// lookups in the hash table and create new IDFIndexEntries or
// TFIndexEntries if they don’t exist and update the frequency
// information for each term-file pair.
#[derive(Debug)]
pub struct Indexer {
    // TODO: FilePathIndex?
    // The FilePathIndex component is a memory intensive component and is responsible for computing a file index from
    // the full file path and storing the index and the full file path into the
    // inverted index in order to be retrieved during search operations. The
    // file content, under the form of a list of extracted tokens, is then indexed by the TFIDFIndex, which is also a memory-intensive component
    // and that indexes the tokens and keeps track of the term frequencies
    // and inverse document frequencies necessary for computing the relevance score.
    pub file: FileIndex,

    pub inner: InvertedIndex,

    pub capacity: usize,

    pub threshold: usize,

    pub tokenizer: Tokenizer,

    pub pipeline: NormalizerPipeline,

    pub counter: TermCounter<String>,
}

impl Indexer {
    pub fn new(
        capacity: usize,
        threshold: usize,
        tokenizer: Tokenizer,
        pipeline: NormalizerPipeline,
    ) -> Self {
        // TODO: Ensure threshold is less than capacity.

        Self {
            file: FileIndex::with_capacity(capacity),
            inner: InvertedIndex::with_capacity(capacity),
            capacity,
            threshold,
            tokenizer,
            pipeline,
            counter: TermCounter::new(),
        }
    }

    // `term_count`: Occurences of term in the document.
    // `path`: Document path.
    // `total_word_count`: Total words in document
    pub fn insert(&mut self, descriptor: Descriptor) {
        // Insert in file index.
        let path = descriptor.path();
        let word_count = descriptor.word_count();
        let index = self.insert_file(path, word_count);

        let mut tokens = self.tokenizer.tokenize(descriptor.document().inner());
        self.pipeline.run(&mut tokens);

        // `index`: file index
        // `freq`: number of times, the term occurs in the file.
        // let tf_entry = TfEntry::new(index, word_frequency);

        // Insert in inverted index.

        for token in tokens {
            self.counter.insert(token.clone());

            let word_frequency = self.word_frequency(&token);

            // TODO: pass reference instead.
            self.insert_entry(token.inner(), word_frequency, index);
        }

        self.counter.reset();
    }

    fn word_frequency(&self, token: &Token) -> usize {
        **self.counter.get(token.as_ref()).unwrap()
    }

    fn insert_file(&mut self, path: &str, word_count: usize) -> usize {
        let entry = FileEntry::new(path.into(), word_count);

        self.file.insert(entry)
    }

    fn insert_entry(&mut self, term: String, word_frequency: usize, file_index: usize) {
        let tf_entry = TfEntry::new(file_index, word_frequency);

        self.inner.insert(term, tf_entry);
    }
}

// TODO: Ensure that same files are not added more than once, maybe use another data structure.
#[derive(Debug)]
pub struct FileIndex {
    inner: Vec<FileEntry>,
}

impl FileIndex {
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(1_000)
    }

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
        self.inner.len() - 1
    }
}

#[derive(Debug)]
pub struct FileEntry {
    path: String,

    // Word count
    count: NonZeroUsize,
}

impl FileEntry {
    pub fn new(path: String, word_count: usize) -> Self {
        Self {
            path,

            // SAFETY:
            // - The value must not be zero.
            // - Empty documents are not indexed.
            count: unsafe { NonZeroUsize::new_unchecked(word_count) },
        }
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

    #[inline]
    pub fn insert(&mut self, term: String, tf_entry: TfEntry) {
        match self.inner.entry(term) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(RefEntry::new(tf_entry));
            }
            Entry::Vacant(entry) => {
                // TODO: Track default capacity and threshold.

                let mut set = HashSet::new();
                set.insert(RefEntry::new(tf_entry));
                entry.insert(IdfEntry { entries: set });

                // let mut map = HashMap::new();
                // map.insert(tf_entry.index, tf_entry.frequency);
                // entry.insert(IdfEntry { entries: map });
            }
        }
    }
}

// SCANNS
// IDFIndexEntry keeps track of the token associated with the entry,
// the number of files that contain the term,
#[derive(Debug, Default)]
pub struct IdfEntry {
    // Token
    // - maybe the key
    // token: String,

    // Number of files that contain the term.
    // - maybe the self.entries.len()
    //
    // file_count: Counter<usize>,
    // Index: Frequency
    // TODO: Maybe, use better data structure for this use-case.
    // entries: HashMap<usize, usize>,
    entries: HashSet<RefEntry>,
    // TODO: Remove threshold from IdfEntry and keep track somewhere else.
    // Limit after which the data will be flushed into the disk.
    // threshold: usize,
}

impl IdfEntry {
    #[inline]
    pub fn with_capacity(capacity: usize, threshold: usize) -> Self {
        Self {
            // entries: HashMap::with_capacity(capacity),
            entries: HashSet::with_capacity(capacity),
        }
    }

    #[inline]
    /// Number of documents containing the term.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn insert(&mut self, entry: RefEntry) {
        // if self.entries.contains(&entry) {
        //     if let Some(entry) = self.entries.get(&entry) {
        //         entry.0.borrow_mut().frequency += 1;
        //     }
        // } else {
        //     self.entries.insert(entry);
        // }

        if let Some(entry) = self.entries.get(&entry) {
            entry.0.borrow_mut().frequency += 1;
        }

        self.entries.insert(entry);

        // match self.entries.entry(entry.index) {
        //     Entry::Occupied(mut frequency) => {
        //         *frequency.get_mut() += 1;
        //     }
        //     Entry::Vacant(_) => {
        //         self.entries.insert(entry.index, entry.frequency);
        //     }
        // }
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
    pub fn new(entry: TfEntry) -> Self {
        Self(RefCell::new(entry))
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
    frequency: usize,
}

impl TfEntry {
    pub fn new(index: usize, frequency: usize) -> Self {
        Self { index, frequency }
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
pub struct AppendCache {
    last_file_index: usize,
    file_token_count: usize,
}

impl AppendCache {
    pub fn new(last_file_index: usize, file_token_count: usize) -> Self {
        Self {
            last_file_index,
            file_token_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::index::{IdfEntry, TfEntry};

    use super::{FileEntry, FileIndex, InvertedIndex};

    const INDEX_CAP: usize = 100;
    const CAPACITY: usize = 100;
    const THRESHOLD: usize = 80;

    #[test]
    fn test_idx_api() {
        // TODO:
        // 1. Document and File Path
        // 2. Insert in file index, get file_path_index
        // 3. Tokenize
        // 4. Normalize Pipeline
        // 5. Descriptor (document and file path)

        // for tokens/terms in document, insert to inverted index
        // 6. Insert in inverted index

        let mut index = InvertedIndex::with_capacity(INDEX_CAP);
        let mut file_index = FileIndex::with_capacity(INDEX_CAP);

        // let file_path_index = file_index.insert(FileEntry::new(path, word_count));
        // let mut tf_entry = TfEntry::new(file_path_index, frequency);

        // index.insert(term, tf_entry);

        let total_doc = 100;
        let document = "he is good boy".to_string();
        let term = "boy".to_string();

        // index.insert(term, )
    }

    // #[test]
    // fn test_index_idf_entry_basic() {
    //     let idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
    //     assert_eq!(idf.entries.capacity(), 100);
    //     // assert_eq!(*idf.file_count, 0);
    //     assert_eq!(idf.entries.len(), 0);
    //     assert_eq!(idf.threshold, 0);
    // }

    // // #[test]
    // // fn test_index_idf_entry_with_file_count() {
    // //     let idf = IdfEntry::new(100).with_file_count(Counter::new(5));
    // //     assert_eq!(*idf.file_count, 5);
    // // }

    // #[test]
    // fn test_index_idf_entry_with_limit() {
    //     let idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
    //     assert_eq!(idf.threshold, 80);
    // }

    // #[test]
    // fn test_index_idf_entry_insert() {
    //     let mut idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
    //     idf.insert(TfEntry::new(1, 3));
    //     assert_eq!(idf.entries.len(), 1);
    //     assert_eq!(idf.entries[0].index, 1);
    //     assert_eq!(idf.entries[0].frequency, 3);
    // }

    // #[test]
    // fn test_index_idf_entry_should_flush() {
    //     let mut idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
    //     (1..=81).for_each(|_| {
    //         idf.insert(TfEntry::default());
    //     });
    //     assert!(idf.should_flush());
    // }

    // #[test]
    // fn test_index_idf_entry_should_not_flush() {
    //     let mut idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
    //     (1..=80).for_each(|_| {
    //         idf.insert(TfEntry::default());
    //     });
    //     assert!(!idf.should_flush());
    // }

    // #[test]
    // fn test_index_idf_entry_range_calculation() {
    //     let mut idf = IdfEntry::with_capacity(CAPACITY, THRESHOLD);
    //     (1..=50).for_each(|_| {
    //         idf.insert(TfEntry::default());
    //     });
    //     assert_eq!(idf.range(), 50);
    // }
}
