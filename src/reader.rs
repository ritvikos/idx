use crate::error::{ConfigError, Error};
use std::{fs::File, path::Path};

pub struct FileReader(File);

impl FileReader {
    // TODO: Direct I/O
    fn new<P: AsRef<Path>>(path: &P) -> Result<Self, Error> {
        match File::open(path) {
            Ok(file) => Ok(FileReader(file)),
            Err(error) => Err(Error::config(ConfigError::File(error.kind()))),
        }
    }
}
