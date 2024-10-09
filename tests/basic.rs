// mod shared;

// use idx::{
//     descriptor::Descriptor,
//     engine::{IdxFacade, Query},
//     normalizer::{case::Lowercase, punctuation::Punctuation, NormalizerPipeline, Stopwords},
//     // tokenizer::{Standard, Tokenizer},
// };

// use idx::tokenizer::{Standard, Tokenizer};

// // use shared::get_test_corpus;

// pub(crate) fn tiny_test_corpus() -> Vec<String> {
//     ["the cat sat on the mat", "the cat sat", "the dog barked"]
//         .iter()
//         .map(|document| document.to_string())
//         .collect::<Vec<_>>()
// }

// #[tokio::test]
// async fn test_indexer_basic_integration() {
//     let corpus = tiny_test_corpus();
//     let tokenizer = Tokenizer::Standard(Standard::new());

//     let mut pipeline = NormalizerPipeline::new();
//     pipeline.insert(Box::new(Lowercase::new()));
//     pipeline.insert(Box::new(Punctuation::new()));
//     pipeline.insert(Box::new(
//         Stopwords::load(&"./src/assets/stopwords/en.txt").unwrap(),
//     ));

//     let mut engine = IdxFacade::new(10, 30, tokenizer.clone(), pipeline);

//     for (idx, document) in corpus.iter().enumerate() {
//         let descriptor = Descriptor::new(format!("path_{}", idx), document.into());
//         engine.insert(descriptor);
//     }

//     // let target = "foxes"; // expected: 7
//     let target = "cat".to_string();
//     let term_doc_count = engine.document_frequency(&target);
//     let total_docs = engine.total_docs(); // expected: 20

//     let query = Query::new(target);
//     let tf = engine.get(query);

//     println!("engine: {engine:#?}");
//     println!("term_doc_count: {term_doc_count:?}");
//     println!("total_docs: {total_docs}");
//     println!("tf: {tf:#?}");
// }
