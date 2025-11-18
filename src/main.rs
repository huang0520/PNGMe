mod args;
mod commands;
mod png_file;

use std::process;

use crate::args::{Cli, Commands};
use clap::Parser;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        process::exit(1);
    }
    process::exit(0);
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Encode {
            file_path,
            chunk_type,
            message,
            output_file,
        } => commands::encode(&file_path, chunk_type, message, output_file.as_deref())?,
        Commands::Decode {
            file_path,
            chunk_type,
        } => match commands::decode(&file_path, chunk_type) {
            Ok(msg) => println!("{}", msg),
            Err(commands::CommandsError::ChunkNotFound(_)) => {
                println!("No chunk with type: {chunk_type}")
            }
            Err(e) => return Err(e.into()),
        },
        Commands::Remove {
            file_path,
            chunk_type,
        } => commands::remove(&file_path, chunk_type)?,
        Commands::Print { file_path } => commands::print(&file_path)?,
    };
    Ok(())
}
