use std::fs;
use std::path::{Path, PathBuf};

use pngme::{Png, PngError};

pub type Result<T> = std::result::Result<T, PngFileError>;

pub struct PngFile {
    path: PathBuf,
    png: Png,
}

#[derive(Debug, thiserror::Error)]
pub enum PngFileError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PNG parsing error: {0}")]
    Png(#[from] PngError),
}

impl PngFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let bytes = fs::read(&path)?;
        let png = Png::try_from(&bytes[..])?;
        Ok(Self { path, png })
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        fs::write(path, &self.png.as_bytes())?;
        Ok(())
    }

    pub fn png(&self) -> &Png {
        &self.png
    }
    pub fn png_mut(&mut self) -> &mut Png {
        &mut self.png
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
}
