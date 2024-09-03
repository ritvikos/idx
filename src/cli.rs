extern crate clap;

use std::num::NonZeroUsize;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub threads: ThreadCommand,
}

#[derive(Debug, Subcommand)]
pub enum ThreadCommand {
    Thread(Thread),
}

#[derive(Debug, Parser)]
pub struct Thread {
    #[arg(default_value_t = NonZeroUsize::new(2).unwrap(), long, short = 'r')]
    pub read: NonZeroUsize,

    #[arg(default_value_t = NonZeroUsize::new(2).unwrap(), long, short = 'i')]
    pub index: NonZeroUsize,

    #[arg(default_value_t = NonZeroUsize::new(2).unwrap(), long, short = 'w')]
    pub write: NonZeroUsize,
}

impl Default for Thread {
    fn default() -> Self {
        Self {
            read: NonZeroUsize::new(2).unwrap(),
            index: NonZeroUsize::new(2).unwrap(),
            write: NonZeroUsize::new(2).unwrap(),
        }
    }
}

impl Thread {
    pub fn new() -> Self {
        Self::default()
    }
}
