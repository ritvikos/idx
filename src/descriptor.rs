use std::path::PathBuf;

use crate::document::Document;

#[derive(Debug)]
pub struct Descriptor {
    path: PathDescriptor,
    document: Document,
}

impl Descriptor {
    pub fn new(path: PathDescriptor, document: Document) -> Self {
        Self { path, document }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut Document {
        &mut self.document
    }
}

#[derive(Debug)]
pub struct PathDescriptor {
    path: PathBuf,
    hash: u64,
}

impl PathDescriptor {
    pub fn new(path: PathBuf, hash: u64) -> Self {
        Self { path, hash }
    }
}

// #[derive(Debug)]
// pub struct Resource {
//     path: PathBuf,
//     document: Document,
// }

// impl Resource {
//     pub fn new(document: Document, path: PathBuf) -> Self {
//         Self { document, path }
//     }

//     pub fn document(&self) -> &Document {
//         &self.document
//     }

//     pub fn document_mut(&mut self) -> &mut Document {
//         &mut self.document
//     }

//     pub fn path(&self) -> &PathBuf {
//         &self.path
//     }

//     pub fn path_mut(&mut self) -> &mut PathBuf {
//         &mut self.path
//     }
// }
