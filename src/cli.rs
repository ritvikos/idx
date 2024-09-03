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
    #[arg(long, short = 'r')]
    pub read: NonZeroUsize,

    #[arg(long, short = 'i')]
    pub index: NonZeroUsize,

    #[arg(long, short = 'w')]
    pub write: NonZeroUsize,
}
