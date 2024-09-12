extern crate clap;
extern crate crossbeam_channel;
extern crate tokio;

use std::{num::NonZeroUsize, path::PathBuf, time::Duration};

use idx::{
    cli::{Cli, ThreadConfig, TokenizerMode},
    descriptor::{Descriptor, PathDescriptor},
    document::Document,
    hash::{CustomHash, CustomHasher, DefaultHasher},
    tokenizer::{Standard, Tokenizer, Whitespace},
};

use clap::Parser;
use crossbeam_channel::{unbounded, Receiver, Sender};
use tokio::{fs::File, io::AsyncReadExt, runtime::Runtime};

#[derive(Debug)]
struct Channel {
    read: (Sender<String>, Receiver<String>),
    index: (Sender<Descriptor>, Receiver<Descriptor>),
    write: (Sender<String>, Receiver<String>),
}

#[derive(Debug, Default)]
struct Config {
    thread: ThreadConfig,
}

#[derive(Debug)]
struct Engine {
    channel: Channel,
    config: Config,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            channel: Channel {
                read: unbounded(),
                index: unbounded(),
                write: unbounded(),
            },
            config: Default::default(),
        }
    }

    pub fn with_thread(mut self, thread: ThreadConfig) -> Self {
        self.config.thread = thread;
        self
    }

    pub fn read(&self) {
        let task = |sender: Sender<Descriptor>, receiver: Receiver<String>| async move {
            let mut hasher = CustomHasher::<DefaultHasher>::new();
            let mut buffer = Vec::with_capacity(100);

            loop {
                while let Ok(path) = receiver.recv() {
                    match File::open(&path).await {
                        Ok(mut file) => match file.read_to_end(&mut buffer).await {
                            Ok(_) => {
                                let buffer = std::mem::take(&mut buffer);
                                let document =
                                    unsafe { Document::from(String::from_utf8_unchecked(buffer)) };

                                let hash = hasher.finalize(&path);
                                hasher.reset();

                                let path = PathDescriptor::new(PathBuf::from(&path), hash);
                                let resource = Descriptor::new(path, document);
                                println!("{resource:?}");
                                sender.send(resource).unwrap();
                            }
                            Err(_) => todo!(),
                        },
                        Err(_) => println!("Error opening the file"),
                    }
                }
            }
        };

        spawn_inner(
            task,
            self.config.thread.read,
            self.channel.index.0.clone(),
            self.channel.read.1.clone(),
        );
    }

    pub fn read_inner<T: CustomHash>(&self, hasher: &mut T, buffer: &mut String) {}

    pub async fn run(&self) {
        self.read();
        // while let Ok(descriptor) = self.channel.index.1.recv() {
        //     println!("{descriptor:?}");
        // }

        // Simulating server.
        loop {
            self.channel
                .read
                .0
                .send(String::from("tests/data/sample.txt"))
                .unwrap();

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}

fn spawn_inner<F, Fut, T, R>(f: F, count: NonZeroUsize, sender: Sender<T>, receiver: Receiver<R>)
where
    F: FnOnce(Sender<T>, Receiver<R>) -> Fut + Send + Clone + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
    T: Send + 'static,
    R: Send + 'static,
{
    (0..count.get()).for_each(|_| {
        let tx = sender.clone();
        let rx = receiver.clone();
        let task = f.clone();

        std::thread::spawn(|| {
            let rt = Runtime::new().expect("Failed to create runtime!");
            rt.block_on(async move { task(tx, rx).await });
        });
    });
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    Engine::new().with_thread(cli.thread).run().await;

    // let thread_config = cli.thrad

    // let thread_config = cli.thread;

    // let (read_tx, read_rx) = unbounded();
    // let (index_tx, index_rx) = unbounded();

    // (0..thread.read.get()).for_each(|_| {
    //     let index_tx = index_tx.clone();
    //     let read_rx = read_rx.clone();

    //     std::thread::spawn(move || {
    //         let rt = Runtime::new().expect("Failed to create runtime in reader threads.");
    //         let mut hasher = CustomHasher::<DefaultHasher>::new();
    //         let mut buffer = Vec::with_capacity(100);

    //         rt.block_on(async move {
    //             loop {
    //                 while let Ok(path) = read_rx.recv() {
    //                     match File::open(&path).await {
    //                         Ok(mut file) => match file.read_to_end(&mut buffer).await {
    //                             Ok(_) => {
    //                                 let buffer = std::mem::take(&mut buffer);
    //                                 let document = unsafe {
    //                                     Document::from(String::from_utf8_unchecked(buffer))
    //                                 };

    //                                 let hash = hasher.finalize(&path);
    //                                 hasher.reset();

    //                                 let path = PathDescriptor::new(PathBuf::from(&path), hash);
    //                                 let resource = Descriptor::new(path, document);
    //                                 index_tx.send(resource).unwrap();
    //                             }
    //                             Err(_) => todo!(),
    //                         },
    //                         Err(_) => todo!(),
    //                     }
    //                     // match
    //                 }
    //             }
    //         });
    //     });
    // });

    // (0..thread.index.get()).for_each(|_| {
    //     let rx = index_rx.clone();
    //     let mut tokenizer = match cli.tokenizer.mode {
    //         TokenizerMode::Standard => Tokenizer::Standard(Standard::new()),
    //         TokenizerMode::Whitespace => Tokenizer::Whitespace(Whitespace::new()),
    //     };

    //     std::thread::spawn(move || loop {
    //         while let Ok(mut descriptor) = rx.recv() {
    //             let tokens = tokenizer.tokenize(descriptor.document_mut());
    //             println!("tokens: {tokens:?}");

    //             // println!("hash: {hash}");
    //             // println!("{descriptor:?}");
    //         }
    //     });
    // });

    // // Main thread will handle server.
    // // Simulating server.
    // loop {
    //     read_tx.send(String::from("tests/data/sample.txt")).unwrap();

    //     tokio::time::sleep(Duration::from_secs(2)).await;
    // }
}

// TODO: Conditional Variable
// TODO: Use shared memory, instead of channels.
