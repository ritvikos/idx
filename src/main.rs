extern crate clap;
extern crate crossbeam_channel;
extern crate tokio;

use std::time::Duration;

use idx::{
    cli::{validate_file, CaseConfig, Cli, NormalizerConfig, TokenizerMode},
    descriptor::Descriptor,
    document::Document,
    hash::CustomHasher,
    index::Index,
    map::TermCounter,
    normalizer::{
        case::{Lowercase, Uppercase},
        punctuation::Punctuation,
        replace::TokenReplacer,
        NormalizerPipeline, Stopwords,
    },
    read::FileReader,
    tokenizer::{Standard, Tokenizer, Whitespace},
};

use clap::Parser;
use crossbeam_channel::{unbounded, Receiver, Sender};
use tokio::{fs::File, io::AsyncReadExt, runtime::Runtime};

const INDEX_CAPACITY: usize = 100;
const THRESHOLD: usize = 80;

#[derive(Clone, Debug)]
struct Channel {
    read: (Sender<String>, Receiver<String>),
    index: (Sender<Descriptor>, Receiver<Descriptor>),
    write: (Sender<String>, Receiver<String>),
}

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

    pub fn index(&self) {}

    fn document(&self, buffer: &mut Vec<u8>) -> Document {
        let buffer = std::mem::take(buffer);
        unsafe { Document::from(String::from_utf8_unchecked(buffer)) }
    }

    fn hash(&self, hasher: &mut CustomHasher, path: &str) -> u64 {
        hasher.reset();
        hasher.finalize(path)
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = cli.init().unwrap();

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

                    let mut stopwords = Vec::new();

                    // TODO: Cleanup
                    match FileReader::new().open(&path).await {
                        Ok(reader) => match reader.read_lines().await {
                            Ok(mut lines) => {
                                while let Ok(Some(word)) = lines.next_line().await {
                                    if !word.is_empty() {
                                        stopwords.push(word);
                                    }
                                }
                            }
                            Err(e) => eprintln!("Error reading lines: {}", e),
                        },
                        Err(error) => panic!("{error:?}"),
                    };

                    pipeline.insert(Box::new(Stopwords::new(stopwords)));
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

    let thread_config = config.thread;

    // -- WIP --
    // handle queues here
    // spawn threads for reading.
    // engine.read(file: String, buffer: &mut String);
    // engine.index(tokenizer: &mut Tokenizer)
    // engine.write()

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
                                    println!("{descriptor:?}");
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
        let mut tokenizer = tokenizer.clone();
        let mut pipeline = pipeline.clone();
        let mut index = Index::new(INDEX_CAPACITY, THRESHOLD);
        let mut counter = TermCounter::new();

        std::thread::spawn(move || loop {
            while let Ok(mut descriptor) = rx.recv() {
                let word_count = descriptor.word_count();
                let mut tokens = tokenizer.tokenize(descriptor.document_mut());
                pipeline.run(&mut tokens);

                for token in tokens {
                    counter.insert(token);
                }

                println!("{counter:?}");
                // tokens.iter().for_each(|term| {
                //     // Number of times term appear in document.
                //     // let frequency =
                //     // frequencies.insert(term);

                //     index.insert(term, descriptor.path(), frequency);
                // });

                // // println!("{descriptor:?}");
                counter.reset();
            }
        });
    });

    // // Main thread will handle server.
    // // Simulating server.
    loop {
        read_tx.send(String::from("tests/data/sample.txt")).unwrap();
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

// TODO: Conditional Variable
// TODO: Use shared memory, instead of channels.
