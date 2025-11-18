use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use pngme::{Chunk, ChunkError, ChunkType, ChunkTypeError, Png, PngError};

pub type Result<T> = std::result::Result<T, CommandsError>;

#[derive(Debug, thiserror::Error)]
pub enum CommandsError {
    #[error("Failed to open the file")]
    IOError(#[from] io::Error),
    #[error("Failed to parse the file")]
    PngError(#[from] PngError),
    #[error("Invalid chunk type")]
    ChunkTypeError(#[from] ChunkTypeError),
    #[error("Invalid chunk format")]
    ChunkError(#[from] ChunkError),
    #[error("Invalid path")]
    InvalidPath,
    #[error("File might be corrupted")]
    CorruptedFile,
}

pub fn encode(file_path: &Path, chunk_type: &str, message: &str, output_file: &Path) -> Result<()> {
    let bytes = fs::read(&file_path)?;
    let mut png = Png::try_from(&bytes[..])?;

    // Create secret chunk and encode it into original file
    let chunk = Chunk::new(
        ChunkType::from_str(chunk_type)?,
        message.as_bytes().to_vec(),
    );
    png.append_chunk(chunk);

    // Write encoded file
    fs::write(&output_file, png.as_bytes())?;
    Ok(())
}

pub fn decode(file_path: &Path, chunk_type: &str) -> Result<Option<String>> {
    let bytes = fs::read(&file_path)?;
    let png = Png::try_from(&bytes[..])?;

    let Some(chunk) = png.chunk_by_type(chunk_type) else {
        return Ok(None);
    };
    Ok(Some(chunk.data_as_str()?.to_string()))
}

pub fn remove(file_path: &Path, chunk_type: &str) -> Result<()> {
    let bytes = fs::read(&file_path)?;
    let mut png = Png::try_from(&bytes[..])?;
    png.remove_first_chunk(chunk_type)?;
    fs::write(file_path, png.as_bytes())?;
    Ok(())
}

pub fn print(file_path: &Path) -> Result<()> {
    let bytes = fs::read(&file_path)?;
    let png = Png::try_from(&bytes[..])?;
    println!("{}", png);
    Ok(())
}

pub fn get_default_output_path(input_path: &Path) -> Result<PathBuf> {
    let parent = input_path
        .parent()
        .ok_or_else(|| CommandsError::InvalidPath)?;
    let stem = input_path
        .file_stem()
        .ok_or_else(|| CommandsError::InvalidPath)?;
    let ext = input_path
        .extension()
        .ok_or_else(|| CommandsError::InvalidPath)?;
    let new_name = format!(
        "{}_encoded.{}",
        stem.to_string_lossy(),
        ext.to_string_lossy()
    );
    Ok(parent.join(new_name))
}
