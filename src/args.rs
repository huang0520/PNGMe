use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Hide a secret message inside a PNG file
    ///
    /// This adds a new chunk to your PNG that contains your message.
    /// Your image will look exactly the same but now carries hidden data.
    ///
    /// Example:
    ///   encode photo.png ruSt "Meet me at midnight"
    Encode {
        /// Path to the PNG image you want to hide a message in
        file_path: PathBuf,

        /// A 4-letter code that identifies your hidden chunk
        ///
        /// Use any 4 letters like: ruSt, hide, note, data
        /// Tip: 'ruSt' is a good choice for secret messages
        ///
        /// WARNING: Don't use standard chunk names like IDAT or IEND
        chunk_type: String,

        /// The secret message you want to hide
        message: String,

        /// Optional: Specify a custom output file path
        ///
        /// If you don't provide this, a new file will be created with "_encode" suffix
        /// Example: input.png becomes input_encode.png
        output_file: Option<PathBuf>,
    },

    /// Find and display a hidden message in a PNG file
    ///
    /// Searches for a specific chunk type and shows the message inside.
    ///
    /// Example:
    ///   decode photo.png ruSt
    Decode {
        /// Path to the PNG image to search
        file_path: PathBuf,

        /// The 4-letter chunk code used when encoding
        ///
        /// This must match exactly what you used to hide the message.
        chunk_type: String,
    },

    /// Remove a hidden message chunk from a PNG file
    ///
    /// WARNING: This deletes the chunk permanently. The message cannot
    /// be recovered. Consider making a backup copy first.
    ///
    /// Example:
    ///   remove photo.png ruSt
    Remove {
        /// Path to the PNG file to clean up
        file_path: PathBuf,

        /// The 4-letter chunk code to delete
        chunk_type: String,
    },

    /// Show all chunks in a PNG file (useful for exploration)
    ///
    /// Lists every chunk in the file to help you find hidden messages
    /// or understand the PNG structure.
    ///
    /// Example:
    ///   print photo.png
    Print {
        /// Path to the PNG file to analyze
        file_path: PathBuf,
    },
}
