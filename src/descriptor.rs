use std::path::PathBuf;

use crate::document::Document;

#[derive(Debug)]
pub struct Descriptor {
    path: PathBuf,
    document: Document,
}

impl Descriptor {
    pub fn new(path: PathBuf, document: Document) -> Self {
        Self { path, document }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut Document {
        &mut self.document
    }
}
