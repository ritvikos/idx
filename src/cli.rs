extern crate clap;
extern crate serde;
extern crate serde_json;

use std::{collections::HashMap, num::NonZeroUsize};

use clap::Parser;
use serde::Deserialize;

use idx::error::{ConfigError, Error};

macro_rules! supported_config_formats {
    ($($name:expr => $format:ident),* $(,)?) => {
        static SUPPORTED_CONFIG_FORMATS: &[(&str, ConfigParser)] = &[
            $(
                ($name, ConfigParser::$format),
            )*
        ];
    }
}

supported_config_formats! {
    "json" => Json,
    "toml" => Toml,
    "yaml" => Yaml,
}

#[derive(Copy, Clone, Debug)]
enum ConfigParser {
    Json,
    Toml,
    Yaml,
}

impl ConfigParser {
    fn from_str(extension: &str) -> Result<Self, Error> {
        for (name, kind) in SUPPORTED_CONFIG_FORMATS {
            if *name == extension {
                return Ok(*kind);
            }
        }

        Err(Error::Config(ConfigError::FileFormat(
            "Unsupported file format".into(),
        )))
    }

    fn parse(&self, buffer: &str) -> Result<Config, Error> {
        match self {
            Self::Json => serde_json::from_str(buffer)
                .map_err(|error| Error::from(ConfigError::Serialization(error.to_string()))),
            Self::Toml => todo!(),
            Self::Yaml => todo!(),
        }
    }
}

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
        self.validate_and_run(|extension| {
            let parser = ConfigParser::from_str(extension)?;
            let buffer = self.read().unwrap();
            let config = parser.parse(&buffer)?;
            Ok(config)
        })
    }

    fn validate_and_run(
        &self,
        f: impl FnOnce(&str) -> Result<Config, Error>,
    ) -> Result<Config, Error> {
        let path = std::path::Path::new(&self.config);

        match path.extension().and_then(|extension| extension.to_str()) {
            Some(extension) => {
                path.try_exists()
                    .map_err(|error| ConfigError::File(error.kind()))?;

                f(extension)
            }
            None => Err(Error::Config(ConfigError::MissingExtension)),
        }
    }

    fn read(&self) -> Result<String, Error> {
        std::fs::read_to_string(&self.config)
            .map_err(|error| Error::from(ConfigError::File(error.kind())))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub thread: ThreadConfig,
    pub tokenizer: TokenizerConfig,
    pub normalizer: Vec<NormalizerConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ThreadConfig {
    pub read: NonZeroUsize,
    pub index: NonZeroUsize,
    pub _write: NonZeroUsize,
}

impl Default for ThreadConfig {
    fn default() -> Self {
        Self {
            read: NonZeroUsize::new(1).unwrap(),
            index: NonZeroUsize::new(2).unwrap(),
            _write: NonZeroUsize::new(1).unwrap(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TokenizerConfig {
    pub mode: TokenizerMode,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenizerMode {
    #[default]
    Standard,
    Whitespace,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizerConfig {
    Case(CaseConfig),
    #[serde(rename = "replacements")]
    Replacer(ReplacerConfig),
    Punctuation(bool),
    // Stemmer(StemmerConfig),
    Stopwords(StopwordsConfig),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseConfig {
    Lowercase,
    Uppercase,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StopwordsConfig {
    pub file: Option<String>,
    pub words: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReplacerConfig {
    pub file: Option<String>,
    pub pairs: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StemmerConfig {}

// // TODO: Tokenizer Config
