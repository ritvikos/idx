extern crate clap;
extern crate serde;
extern crate serde_json;

use std::{collections::HashMap, num::NonZeroUsize};

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::error::{ConfigError, Error};

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    // #[command(flatten)]
    // pub thread: ThreadConfig,

    // #[command(flatten)]
    // pub tokenizer: TokenizerConfig,

    // stopwords = txt / value
    // stemming  =
    // replacer  = json
    // case      = value
    // pub normalizer: PathBuf,
    #[arg(long, value_name = "FILE")]
    pub config: String,
}

impl Cli {
    pub fn init(&self) -> Result<Config, Error> {
        self.validate(".json")?;
        let buffer = self.read()?;
        self.parse(&buffer)
    }

    fn validate(&self, extension: &str) -> Result<(), Error> {
        if !self.config.ends_with(extension) {
            return Err(Error::from(ConfigError::File(
                std::io::ErrorKind::InvalidInput,
            )));
        }

        std::path::Path::new(&self.config)
            .try_exists()
            .map_err(|error| ConfigError::File(error.kind()))?;

        Ok(())
    }

    fn read(&self) -> Result<String, Error> {
        std::fs::read_to_string(&self.config)
            .map_err(|error| Error::from(ConfigError::File(error.kind())))
    }

    fn parse(&self, buffer: &str) -> Result<Config, Error> {
        serde_json::from_str(buffer)
            .map_err(|error| Error::from(ConfigError::Serialization(error.to_string())))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub thread: ThreadConfig,
    pub tokenizer: TokenizerConfig,
    pub normalizer: Vec<NormalizerConfig>,
}

// #[derive(Clone, Debug, Parser)]
// #[command(group(
//     ArgGroup::new("thread").required(false).multiple(true),
// ))]
#[derive(Debug, Deserialize, Serialize)]
pub struct ThreadConfig {
    // #[arg(long = "thread-read", short = 'r', group = "thread")]
    pub read: NonZeroUsize,

    // #[arg(long = "thread-index", short = 'i', group = "thread")]
    pub index: NonZeroUsize,

    // #[arg(long = "thread-write", short = 'w', group = "thread")]
    pub write: NonZeroUsize,
}

impl ThreadConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ThreadConfig {
    fn default() -> Self {
        Self {
            read: NonZeroUsize::new(1).unwrap(),
            index: NonZeroUsize::new(2).unwrap(),
            write: NonZeroUsize::new(1).unwrap(),
        }
    }
}

// #[derive(Clone, Debug, Default, Parser)]
// #[command(group(
// ArgGroup::new("tokenizer").required(false).multiple(true),
// ))]
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TokenizerConfig {
    // #[arg(long = "tokenizer-mode", group = "tokenizer")]
    pub mode: TokenizerMode,
}

impl TokenizerConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

// #[derive(Debug, Default, Clone, ValueEnum)]
#[derive(Debug, Default, Deserialize, Serialize)]
pub enum TokenizerMode {
    #[default]
    #[serde(rename = "standard")]
    Standard,

    #[serde(rename = "whitespace")]
    Whitespace,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum NormalizerConfig {
    #[serde(rename = "case")]
    Case(CaseConfig),

    #[serde(rename = "stopwords")]
    Stopwords(StopwordsConfig),

    #[serde(rename = "replacements")]
    Replacer(ReplacerConfig),

    #[serde(rename = "punctuation")]
    Punctuation(bool),
    // #[serde(rename = "stemmer")]
    // Stemmer(StemmerConfig),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CaseConfig {
    #[serde(rename = "lowercase")]
    Lowercase,

    #[serde(rename = "uppercase")]
    Uppercase,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StopwordsConfig {
    pub file: Option<String>,
    pub words: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReplacerConfig {
    pub file: Option<String>,
    pub pairs: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StemmerConfig {}

pub fn validate_file(file: &str, extension: &str) -> Result<(), Error> {
    if !file.ends_with(extension) {
        return Err(Error::from(ConfigError::File(
            std::io::ErrorKind::InvalidInput,
        )));
    }

    std::path::Path::new(file)
        .try_exists()
        .map_err(|error| ConfigError::File(error.kind()))?;

    Ok(())
}

// // TODO: Tokenizer Config
