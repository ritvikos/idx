extern crate clap;

use std::num::NonZeroUsize;

use clap::{ArgGroup, Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub thread: ThreadConfig,

    #[command(flatten)]
    pub tokenizer: TokenizerConfig,
}

#[derive(Debug, Parser)]
#[command(group(
    ArgGroup::new("thread").required(false).multiple(true),
))]
pub struct ThreadConfig {
    #[arg(long = "thread-read", short = 'r', group = "thread")]
    pub read: NonZeroUsize,

    #[arg(long = "thread-index", short = 'i', group = "thread")]
    pub index: NonZeroUsize,

    #[arg(long = "thread-write", short = 'w', group = "thread")]
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

#[derive(Debug, Default, Parser)]
#[command(group(
    ArgGroup::new("tokenizer").required(false).multiple(true),
))]
pub struct TokenizerConfig {
    #[arg(long = "tokenizer-mode", group = "tokenizer")]
    pub mode: TokenizerMode,
}

impl TokenizerConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default, Clone, ValueEnum)]
pub enum TokenizerMode {
    #[default]
    Standard,
    Whitespace,
}

// // TODO: Tokenizer Config
