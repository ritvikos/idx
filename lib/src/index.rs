//! The [`CoreIndex`] is the core component that handles indexing operations.
//! Internally, it manages inverted-index, file-index, term-counter.
//!
//! Inverted Index
//! The [`InvertedIndex`] handles the core inverted index data structure and
//! exposes methods to perform operations.

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

use crate::{
    core::{FileIndex, InvertedIndex, TermCounter, TfIdf},
    reader::{IndexReader, ReaderContext},
    token::Tokens,
    writer::{FileEntryState, IndexWriter, WriterContext},
};

/// # Indexer
///
/// Currently, a single-threaded data structure implementation that internally utilizes
/// multithreading and SIMD for data-level parallelism,
/// optimizing throughput and tail latency.
///
/// The current strategy utilizes a single-threaded, thread-local indexer
/// and perform a merge operation to generate a global index view.
#[derive(Debug)]
pub struct Index {
    // The FilePathIndex component is a memory intensive component and is responsible for computing a file index from
    // the full file path and storing the index and the full file path into the
    // inverted index in order to be retrieved during search operations. The
    // file content, under the form of a list of extracted tokens, is then indexed by the TFIDFIndex, which is also a memory-intensive component
    // and that indexes the tokens and keeps track of the term frequencies
    // and inverse document frequencies necessary for computing the relevance score.
    pub core: CoreIndex,

    pub capacity: usize,

    pub threshold: usize,
}

pub trait Indexer {
    fn new(capacity: usize, threshold: usize) -> Self;
    fn insert(&mut self, path: String, word_count: usize, tokens: &mut Tokens);
    fn get(&self, term: &str) -> Option<Vec<TfIdf>>;
}

impl Indexer for Index {
    /// Creates a new instance of `Index`
    fn new(capacity: usize, threshold: usize) -> Self {
        // TODO: Ensure threshold is less than capacity.

        Self {
            core: CoreIndex::with_capacity(capacity),
            capacity,
            threshold,
        }
    }

    // TODO:
    // File entry is stored.
    // What if system outage happens at this stage.
    // The file store and inverted index won't sync,
    // ends up with potentially corrupted state.

    // Approach (runtime overhead):
    // Maintain write-ahead logs to re-construct the
    // core index in correct state.
    fn insert(&mut self, path: String, word_count: usize, tokens: &mut Tokens) {
        let writer = self.core.writer();
        let file_entry = WriterContext::<FileEntryState>::new(writer);
        let mut term_entry = file_entry.entry(path, word_count);

        tokens.for_each_mut(|token| {
            term_entry.insert_term_with(|| std::mem::take(token));
        });

        term_entry.reset_counter()
    }

    fn get(&self, term: &str) -> Option<Vec<TfIdf>> {
        let reader = self.core.reader();
        let ctx = ReaderContext::new(reader);

        ctx.get_entry_with(term, |idf_entry| {
            debug_assert!(idf_entry.count() > 0);

            idf_entry.iter_with(|ref_entry| {
                let index = ref_entry.get_index();
                let frequency = *ref_entry.get_frequency();
                let count = ctx.count(index);

                // Always greater than zero, empty documents are not indexed.
                debug_assert!(count > 0);

                let total_documents = ctx.total_documents();
                let document_frequency = idf_entry.count();

                let tf = self.tf(frequency, count);
                let idf = self.idf(total_documents, document_frequency);
                let tfidf = tf * idf;

                TfIdf::new(index, tfidf)
            })
        })
    }
}

impl Index {
    // FIXME: Need more robust conversion mechanism, as it can overflow.
    pub fn tf(&self, frequency: usize, word_count: usize) -> f32 {
        frequency as f32 / word_count as f32
    }

    pub fn idf(&self, total_documents: usize, document_frequency: usize) -> f32 {
        (total_documents as f32 / document_frequency as f32).log10()
    }

    /// Number of documents containing the term.
    #[inline]
    pub fn document_frequency(&self, term: &str) -> Option<usize> {
        let reader = self.core.reader();
        let ctx = ReaderContext::new(reader);
        ctx.document_frequency(term)
    }

    /// Number of indexed documents.
    #[inline]
    pub fn total_docs(&self) -> usize {
        let reader = self.core.reader();
        let ctx = ReaderContext::new(reader);
        ctx.total_documents()
    }
}

#[derive(Debug)]
pub struct CoreIndex {
    store: FileIndex,
    index: InvertedIndex,
    count: TermCounter,
}

impl CoreIndex {
    /// Creates a new instance of [`CoreIndex`]
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            store: FileIndex::with_capacity(capacity),
            index: InvertedIndex::with_capacity(capacity),
            count: TermCounter::new(),
        }
    }

    /// The `CoreIndex` never interacts with external environment
    /// directly for READ operations.
    ///
    /// [`IndexReader`] provides READ access to the index.
    ///
    /// # See Also
    ///
    /// - [`IndexWriter`]: Provides WRITE access to the index.
    pub fn reader(&self) -> IndexReader {
        IndexReader::new(&self.store, &self.index, &self.count)
    }

    /// The `CoreIndex` never interacts with external environment
    /// directly for WRITE operations.
    ///
    /// [`IndexWriter`] provides WRITE access to the index.
    ///
    /// # See Also
    ///
    /// - [`IndexReader`]: Provides READ access to the index.
    pub fn writer(&mut self) -> IndexWriter {
        IndexWriter::new(&mut self.store, &mut self.index, &mut self.count)
    }
}

#[cfg(test)]
mod tests {

    // const INDEX_CAP: usize = 100;
    // const CAPACITY: usize = 100;
    // const THRESHOLD: usize = 80;

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

        // let mut indexer = CoreIndex::with_capacity(20);
        // let mut writer = indexer.writer();
        // writer.insert_counter("t".into());

        // let reader = indexer.reader();
        // let entries = reader.get_term_entries("t");

        // let mut index = InvertedIndex::with_capacity(INDEX_CAP);
        // let mut file_index = FileIndex::with_capacity(INDEX_CAP);

        // let file_path_index = file_index.insert(FileEntry::new(path, word_count));
        // let mut tf_entry = TfEntry::new(file_path_index, frequency);

        // index.insert(term, tf_entry);

        // let total_doc = 100;
        // let document = "he is good boy".to_string();
        // let term = "boy".to_string();

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
