extern crate clap;
extern crate crossbeam_channel;
extern crate tokio;

use std::time::Duration;

use idx::{
    cli::{validate_file, CaseConfig, Cli, NormalizerConfig, TokenizerMode},
    descriptor::Descriptor,
    document::Document,
    engine::{IdxFacade, Query, SearchContext},
    normalizer::{
        case::{Lowercase, Uppercase},
        punctuation::Punctuation,
        replace::TokenReplacer,
        NormalizerPipeline, Stopwords,
    },
    tokenizer::{Standard, Tokenizer, Whitespace},
};

use clap::Parser;
use crossbeam_channel::unbounded;
use tokio::{fs::File, io::AsyncReadExt, runtime::Runtime};

const INDEX_CAPACITY: usize = 100;
const THRESHOLD_CAPACITY: usize = 80;

#[derive(Clone, Debug)]
struct Engine {}

impl Engine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read(&self, path: String, buffer: &mut Vec<u8>) -> Descriptor {
        let document = self.document(buffer);
        Descriptor::new(path, document)
    }

    fn document(&self, buffer: &mut Vec<u8>) -> Document {
        let buffer = std::mem::take(buffer);
        unsafe { Document::from(String::from_utf8_unchecked(buffer)) }
    }
}

// TODO: Create high level API.
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config = cli.init().unwrap();
    let thread_config = config.thread;

    let engine = Engine::new();
    let mut pipeline = NormalizerPipeline::new();

    let tokenizer = match config.tokenizer.mode {
        TokenizerMode::Standard => Tokenizer::Standard(Standard::new()),
        TokenizerMode::Whitespace => Tokenizer::Whitespace(Whitespace::new()),
    };

    for field in config.normalizer {
        match field {
            NormalizerConfig::Case(case) => match case {
                CaseConfig::Lowercase => {
                    pipeline.insert(Box::new(Lowercase::new()));
                }
                CaseConfig::Uppercase => {
                    pipeline.insert(Box::new(Uppercase::new()));
                }
            },

            NormalizerConfig::Stopwords(config) => {
                if let Some(path) = config.file {
                    if let Err(error) = validate_file(&path, ".txt") {
                        panic!("{error:?}");
                    };

                    match Stopwords::load(&path).await {
                        Ok(stopwords) => pipeline.insert(Box::new(stopwords)),
                        Err(error) => panic!("{error:?}"),
                    };

                    continue;
                };

                if let Some(words) = config.words {
                    pipeline.insert(Box::new(Stopwords::new(words)));
                    continue;
                }

                eprintln!("Error: No valid stopwords file or words provided");
                return;
            }

            NormalizerConfig::Replacer(config) => {
                if let Some(file) = config.file {
                    if let Err(error) = validate_file(&file, ".json") {
                        panic!("{error:?}");
                    };

                    // decide!("txt file", "json file", "support both");
                    // read file line by line

                    continue;
                };

                if let Some(pairs) = config.pairs {
                    let replacer = TokenReplacer::new(pairs);
                    pipeline.insert(Box::new(replacer));
                    continue;
                }

                eprintln!("Error: No valid normalizers file or pairs provided");
                return;
            }

            NormalizerConfig::Punctuation(status) => {
                if status {
                    pipeline.insert(Box::new(Punctuation::new()));
                }
            }
        };
    }

    println!("{pipeline:#?}");

    let (read_tx, read_rx) = unbounded();
    let (index_tx, index_rx) = unbounded();

    (0..thread_config.read.get()).for_each(|_| {
        let index_tx = index_tx.clone();
        let read_rx = read_rx.clone();
        let context = engine.clone();

        std::thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create runtime in reader threads.");
            let mut buffer = Vec::with_capacity(800);

            rt.block_on(async move {
                loop {
                    while let Ok(path) = read_rx.recv() {
                        match File::open(&path).await {
                            Ok(mut file) => match file.read_to_end(&mut buffer).await {
                                Ok(_) => {
                                    let descriptor = context.read(path, &mut buffer);
                                    // println!("{descriptor:?}");
                                    index_tx.send(descriptor).unwrap();
                                }
                                Err(_) => todo!(),
                            },
                            Err(_) => todo!(),
                        }
                    }
                }
            });
        });
    });

    (0..thread_config.index.get()).for_each(|_| {
        let rx = index_rx.clone();

        let mut coordinator = IdxFacade::new(
            INDEX_CAPACITY,
            THRESHOLD_CAPACITY,
            tokenizer.clone(),
            pipeline.clone(),
        );

        // let mut indexer = Indexer::new(INDEX_CAPACITY, THRESHOLD_CAPACITY);
        let value = pipeline.clone();

        std::thread::spawn(move || loop {
            while let Ok(descriptor) = rx.recv() {
                // indexer.insert(descriptor);
                // println!("{indexer:#?}");

                coordinator.insert(descriptor);
                // println!("{coordinator:#?}");

                // let term_doc_count = coordinator.term_doc_count("doe");
                // let total_docs = coordinator.total_docs();
                let ctx =
                    SearchContext::new(&coordinator.index, coordinator.tokenizer.clone(), &value);

                let term_doc_count = ctx.doc_term_frequency("doe");
                let total_docs = ctx.total_docs();

                // println!("{:#?}", coordinator.indexer.core.store);
                // println!("frequency of word in query: {}");

                println!("Number of documents containing the term: {term_doc_count:?}");
                println!("Total documents: {total_docs}");
            }
        });
    });

    // // Main thread will handle server.
    // // Simulating server.
    loop {
        read_tx.send(String::from("tests/data/sample.txt")).unwrap();
        read_tx
            .send(String::from("tests/data/sample2.txt"))
            .unwrap();

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

// TODO: Conditional Variable
// TODO: Use shared memory, instead of channels.
