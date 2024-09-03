extern crate crossbeam;
extern crate tokio;

use std::{path::PathBuf, sync::Arc, time::Duration};

use crossbeam::queue::SegQueue;
use tokio::{runtime::Runtime, sync::mpsc};

#[tokio::main]
async fn main() {
    let queue = Arc::new(SegQueue::<PathBuf>::new());
    let (tx, mut rx) = mpsc::unbounded_channel();

    {
        let queue = queue.clone();

        std::thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create runtime in reader threads.");

            rt.block_on(async move {
                loop {
                    while let Some(path) = queue.pop() {
                        match tokio::fs::read_to_string(path).await {
                            Ok(buffer) => {
                                tx.send(buffer).unwrap();
                            }
                            Err(_) => todo!(),
                        }
                    }

                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            });
        })
    };

    std::thread::spawn(move || {
        let rt = Runtime::new().expect("Failed to create runtime in processing threads.");

        rt.block_on(async move {
            loop {
                while let Some(buffer) = rx.recv().await {
                    println!("buffer: {buffer}");
                }
            }
        });
    });

    // Main thread will handle server.
    // Simulating server.
    loop {
        queue.push(PathBuf::from("tests/data/sample.txt"));
        std::thread::sleep(Duration::from_secs(2));
    }
}

// TODO: Conditional Variable
