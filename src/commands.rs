use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::png_file::{PngFile, PngFileError};
use pngme::{Chunk, ChunkError, ChunkType, ChunkTypeError, PngError};

pub type Result<T> = std::result::Result<T, CommandsError>;

#[derive(Debug, thiserror::Error)]
pub enum CommandsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PNG file error: {0}")]
    PngFile(#[from] PngFileError),
    #[error("PNG error: {0}")]
    Png(#[from] PngError),
    #[error("Chunk error: {0}")]
    Chunk(#[from] ChunkError),
    #[error("Chunk type error: {0}")]
    ChunkType(#[from] ChunkTypeError),
    #[error("Chunk not found: {0}")]
    ChunkNotFound(String),
}

pub fn encode(
    file_path: impl AsRef<Path>,
    chunk_type: &str,
    message: &str,
    output_file: Option<impl AsRef<Path>>,
) -> Result<()> {
    let mut png_file = PngFile::load(&file_path)?;

    // Create secret chunk and encode it into original file
    let chunk = Chunk::new(
        ChunkType::from_str(chunk_type)?,
        message.as_bytes().to_vec(),
    );
    png_file.png_mut().append_chunk(chunk);

    // Write encoded file
    let output = output_file
        .map(|p| p.as_ref().to_path_buf())
        .unwrap_or_else(|| default_output_path(&file_path, "encoded"));
    png_file.save(&output)?;
    Ok(())
}

pub fn decode(file_path: impl AsRef<Path>, chunk_type: &str) -> Result<String> {
    let png_file = PngFile::load(&file_path)?;

    Ok(png_file
        .png()
        .chunk_by_type(chunk_type)
        .ok_or_else(|| CommandsError::ChunkNotFound(chunk_type.to_string()))?
        .data_as_str()?
        .to_string())
}

pub fn remove(file_path: impl AsRef<Path>, chunk_type: &str) -> Result<()> {
    let mut png_file = PngFile::load(&file_path)?;

    png_file.png_mut().remove_first_chunk(chunk_type)?;
    png_file.save(&file_path)?;
    Ok(())
}

pub fn print(file_path: &Path) -> Result<()> {
    let png_file = PngFile::load(file_path)?;
    println!("{}", png_file.png());
    Ok(())
}

pub fn default_output_path(input_path: impl AsRef<Path>, suffix: &str) -> PathBuf {
    let input_path = input_path.as_ref();
    let parent = input_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = input_path.file_stem().unwrap_or_default();
    let ext = input_path.extension().unwrap_or_default();

    let new_name = format!(
        "{}_{}.{}",
        stem.to_string_lossy(),
        suffix,
        ext.to_string_lossy()
    );
    parent.join(new_name)
}
