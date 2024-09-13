extern crate clap;
extern crate crossbeam_channel;
extern crate tokio;

use std::{path::PathBuf, time::Duration};

use idx::{
    cli::{Cli, ThreadConfig, TokenizerConfig, TokenizerMode},
    descriptor::{Descriptor, PathDescriptor},
    document::Document,
    hash::{CustomHasher, DefaultHasher},
    tokenizer::{Standard, Tokenizer, Whitespace},
};

use clap::Parser;
use crossbeam_channel::{unbounded, Receiver, Sender};
use tokio::{fs::File, io::AsyncReadExt, runtime::Runtime};

#[derive(Clone, Debug)]
struct Channel {
    read: (Sender<String>, Receiver<String>),
    index: (Sender<Descriptor>, Receiver<Descriptor>),
    write: (Sender<String>, Receiver<String>),
}

#[derive(Clone, Debug, Default)]
struct Config {
    thread: ThreadConfig,
    tokenizer: TokenizerConfig,
}

#[derive(Clone, Debug)]
struct Engine {}

impl Engine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read(
        &self,
        path: String,
        hasher: &mut CustomHasher,
        buffer: &mut Vec<u8>,
    ) -> Descriptor {
        let document = self.document(buffer);
        let hash = self.hash(hasher, &path);

        let path_descriptor = PathDescriptor::new(PathBuf::from(path), hash);
        Descriptor::new(path_descriptor, document)
    }

    fn document(&self, buffer: &mut Vec<u8>) -> Document {
        let buffer = std::mem::take(buffer);
        unsafe { Document::from(String::from_utf8_unchecked(buffer)) }
    }

    fn hash(&self, hasher: &mut CustomHasher, path: &str) -> u64 {
        hasher.reset();
        hasher.finalize(path)
    }

    fn index(&self) {}
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let engine = Engine::new();
    let thread_config = cli.thread;

    let tokenizer = match cli.tokenizer.mode {
        TokenizerMode::Standard => Tokenizer::Standard(Standard::new()),
        TokenizerMode::Whitespace => Tokenizer::Whitespace(Whitespace::new()),
    };

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
            let mut hasher = CustomHasher::<DefaultHasher>::new();
            let mut buffer = Vec::with_capacity(800);

            rt.block_on(async move {
                loop {
                    while let Ok(path) = read_rx.recv() {
                        match File::open(&path).await {
                            Ok(mut file) => match file.read_to_end(&mut buffer).await {
                                Ok(_) => {
                                    let descriptor = context.read(path, &mut hasher, &mut buffer);
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

        std::thread::spawn(move || loop {
            while let Ok(mut descriptor) = rx.recv() {
                let tokens = tokenizer.tokenize(descriptor.document_mut());
                println!("tokens: {tokens:?}");

                // println!("hash: {hash}");
                // println!("{descriptor:?}");
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
