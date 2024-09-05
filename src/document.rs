use std::{
    fmt::{Display, Formatter},
    ops::{Deref, DerefMut},
    path::PathBuf,
};

#[derive(Debug)]
pub struct Resource {
    path: PathBuf,
    document: Document,
}

impl Resource {
    pub fn new(document: Document, path: PathBuf) -> Self {
        Self { document, path }
    }
}

#[derive(Debug)]
pub struct Document(String);

impl Document {
    pub fn new(doc: String) -> Self {
        Self(doc)
    }

    pub fn get(&self) -> &String {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

impl From<String> for Document {
    fn from(buffer: String) -> Self {
        Document(buffer)
    }
}

impl Deref for Document {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl DerefMut for Document {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Document {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
