extern crate clap;
extern crate crossbeam_channel;
extern crate tokio;

use std::{path::PathBuf, time::Duration};

use idx::{
    cli::{Cli, ThreadCommand},
    document::{Document, Resource},
};

use clap::Parser;
use crossbeam_channel::unbounded;
use tokio::{fs::File, io::AsyncReadExt, runtime::Runtime};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let thread = match cli.threads {
        ThreadCommand::Thread(config) => config,
    };

    let (read_tx, read_rx) = unbounded();
    let (index_tx, index_rx) = unbounded();

    {
        (0..thread.read.get()).into_iter().for_each(|_| {
            let index_tx = index_tx.clone();
            let read_rx = read_rx.clone();

            std::thread::spawn(move || {
                let rt = Runtime::new().expect("Failed to create runtime in reader threads.");
                let mut buffer = Vec::with_capacity(100);

                rt.block_on(async move {
                    loop {
                        while let Ok(path) = read_rx.recv() {
                            match File::open(&path).await {
                                Ok(mut file) => match file.read_to_end(&mut buffer).await {
                                    Ok(_) => {
                                        let buffer = std::mem::take(&mut buffer);
                                        let document = unsafe {
                                            Document::from(String::from_utf8_unchecked(buffer))
                                        };
                                        let resource = Resource::new(document, path);
                                        index_tx.send(resource).unwrap();
                                    }
                                    Err(_) => todo!(),
                                },
                                Err(_) => todo!(),
                            }
                            // match
                        }
                    }
                });
            });
        });
    };

    (0..thread.index.get()).into_iter().for_each(|_| {
        let rx = index_rx.clone();

        std::thread::spawn(move || loop {
            while let Ok(resource) = rx.recv() {
                println!("resource: {resource:?}");
            }
        });
    });

    // Main thread will handle server.
    // Simulating server.
    loop {
        read_tx
            .send(PathBuf::from("tests/data/sample.txt"))
            .unwrap();

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

// TODO: Conditional Variable
