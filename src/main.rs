mod args;
mod commands;

use crate::args::{Cli, Commands};
use clap::Parser;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Encode {
            file_path,
            chunk_type,
            message,
            output_file,
        } => {
            let output_file = match output_file {
                Some(path) => path.clone(),
                None => commands::get_default_output_path(file_path)?,
            };
            commands::encode(file_path, chunk_type, message, &output_file)?
        }
        Commands::Decode {
            file_path,
            chunk_type,
        } => println!(
            "{:#?}",
            commands::decode(file_path, chunk_type)?
                .unwrap_or_else(|| format!("No chunk with type: {}", chunk_type))
        ),
        Commands::Remove {
            file_path,
            chunk_type,
        } => commands::remove(file_path, chunk_type)?,
        Commands::Print { file_path } => commands::print(file_path)?,
    };
    Ok(())
}
