extern crate tokio;

use std::path::{Path, PathBuf};

use idx::error::{ConfigError, Error, IoError};

use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncReadExt, BufReader, Lines},
};

#[derive(Debug, Default)]
pub struct FileReader {
    inner: Option<File>,
    path: PathBuf,
}

impl FileReader {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use = "File path must be set"]
    pub async fn open<P: AsRef<Path>>(&mut self, path: P) -> Result<&mut Self, Error> {
        let path_buf = path.as_ref().to_path_buf();

        match File::open(&path_buf).await {
            Ok(file) => {
                self.inner = Some(file);
                self.path = path_buf;
                Ok(self)
            }
            Err(error) => Err(ConfigError::File(error.kind()).into()),
        }
    }

    pub async fn to_buf_reader(&mut self) -> Result<BufReader<File>, Error> {
        if let Some(file) = self.inner.take() {
            Ok(BufReader::new(file))
        } else {
            let file = File::open(&self.path)
                .await
                .map_err(|error| Error::from(ConfigError::File(error.kind())))?;

            Ok(BufReader::new(file))
        }
    }

    pub async fn read_lines(&mut self) -> Result<Lines<BufReader<File>>, Error> {
        let reader = self.to_buf_reader().await?;
        Ok(reader.lines())
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
                .map_err(|error| panic!("{:?}", IoError::File(error.kind()))),

            None => Err(IoError::Reader(std::io::ErrorKind::InvalidInput).into()),
        }
    }
}

// TODO: Direct I/O

#[cfg(test)]
mod tests {
    use crate::read::FileReader;

    #[tokio::test]
    async fn test_reader_file_open() {
        let path = "tests/data/sample.txt".to_string();

        let mut buffer = String::new();
        let mut reader = FileReader::new();

        reader.open(path).await.unwrap();
        reader.read_into(&mut buffer).await.unwrap();
    }
}
