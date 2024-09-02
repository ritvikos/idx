use std::path::{Path, PathBuf};

use crate::error::{ConfigError, Error, IoError};

use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug, Default)]
pub struct FileReader {
    inner: Option<File>,
    path: PathBuf,
}

impl FileReader {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn open<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let path_buf = path.as_ref().to_path_buf();

        match File::open(&path_buf).await {
            Ok(file) => {
                self.inner = Some(file);
                self.path = path_buf;
                Ok(())
            }
            Err(error) => Err(ConfigError::File(error.kind()).into()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn read_into(&mut self, buffer: &mut String) -> Result<(), Error> {
        match &mut self.inner {
            Some(reader) => reader
                .read_to_string(buffer)
                .await
                .map(|_| ())
                .map_err(|error| Err(IoError::File(error.kind())).unwrap()),

            None => Err(IoError::Reader(std::io::ErrorKind::InvalidInput).into()),
        }
    }
}

// TODO: Direct I/O

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        sync::{atomic::AtomicBool, Arc, Condvar},
    };

    use crate::read::FileReader;

    use crossbeam::queue::SegQueue;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_reader_file_open() {
        let path = "tests/data/sample.txt".to_string();

        let mut buffer = String::new();
        let mut reader = FileReader::new();

        reader.open(path).await.unwrap();
        reader.read_into(&mut buffer).await.unwrap();
    }

    #[tokio::test]
    async fn test_reader_file_multi_thread() {
        const READ_THREADS: usize = 2;

        let (tx, mut rx) = mpsc::unbounded_channel();
        let queue = Arc::new(SegQueue::new());
        let (lock, cvar) = (AtomicBool::new(false), Arc::new(Condvar::new()));

        queue.push(PathBuf::from("tests/data/html.txt"));
        queue.push(PathBuf::from("tests/data/sample.txt"));

        (0..READ_THREADS).for_each(|_| {
            let tx = tx.clone();
            let queue = queue.clone();

            std::thread::spawn(move || {
                if let Some(path) = queue.pop() {
                    tokio::spawn(async move {
                        tx.send(path).unwrap();
                    });
                }
            });
        });

        while let Some(path) = rx.recv().await {
            println!("{path:?}");
        }

        // tokio::spawn(async { while let Some(path) = rx.recv().await {} });

        // rayon::scope(|s| {
        //     (0..READ_THREADS).into_iter().for_each(|_| {
        //         s.spawn(|_| {
        //             if let Some(path) = queue.pop() {
        //                 println!("path: {:?}", path);
        //             }
        //         })
        //     })
        // });
    }
}
