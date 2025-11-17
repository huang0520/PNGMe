mod args;
mod chunk;
mod chunk_type;
// mod commands;
mod png;
mod utils;

use clap::Parser;

use crate::args::Cli;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let cli = Cli::parse();
    todo!()
}
