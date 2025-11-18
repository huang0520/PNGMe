use std::fmt;

use crate::chunk_type::{ChunkType, ChunkTypeError};
use crc::{CRC_32_ISO_HDLC, Crc};

/// CRC-32 algorithm instance used for PNG chunk verification (ISO/HDLC standard).
/// This is the standard CRC-32 algorithm specified in the PNG specification (ISO 3309).
pub const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

/// Specialized `Result` type for `Chunk` operations.
/// This type is used throughout the chunk module for any operation that can produce a `ChunkError`.
pub type Result<T> = std::result::Result<T, ChunkError>;

/// A PNG chunk as defined in the PNG specification (ISO/IEC 15948).
///
/// PNG chunks are the fundamental building blocks of PNG files, consisting of:
/// - A 4-byte length field (big-endian)
/// - A 4-byte chunk type identifier
/// - Variable-length data
/// - A 4-byte CRC checksum
///
/// # Examples
///
/// Basic usage:
/// ```ignore
/// let chunk_type = ChunkType::try_from(RuSt)?;
/// let data = This is where your secret message will be!.as_bytes().to_vec();
/// let chunk = Chunk::new(chunk_type, data);
/// ```
///
/// # Fields
///
/// - `chunk_type`: The type of chunk (e.g., IHDR, IDAT, tEXt, etc.)
/// - `data`: The chunk's payload data
/// - `crc`: CRC-32 checksum calculated over chunk type and data
pub struct Chunk {
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

/// Errors that can occur when parsing or constructing a PNG chunk.
///
/// This enum covers all failure modes for chunk operations, including
/// parsing from raw bytes and CRC verification failures.
#[derive(Debug, thiserror::Error)]
pub enum ChunkError {
    /// Returned when the input byte slice doesn't contain enough data
    /// to parse a complete chunk field.
    ///
    /// # Fields
    ///
    /// - `position`: The byte offset in the input where the read was attempted
    /// - `required`: The number of bytes needed
    /// - `actual`: The number of bytes actually available
    #[error("Insufficient data at position {position}: need {required} bytes, got {actual}")]
    NotEnoughBytes {
        /// Byte position in the input where the read was attempted
        position: usize,
        required: usize,
        actual: usize,
    },

    /// Returned when the chunk data length exceeds the PNG specification limit
    /// of 2^31 - 1 bytes.
    ///
    /// # Fields
    ///
    /// - `size`: The invalid data size that was encountered
    #[error("Chunk data is larger than PNG Spec: size {size}")]
    TooLarge {
        /// The size that exceeds the specification limit
        size: usize,
    },

    /// Returned when the chunk type contains invalid characters or fails
    /// to meet PNG chunk type requirements.
    #[error("Invalid chunk type")]
    InvalidChunkType(#[from] ChunkTypeError),

    /// Returned when the CRC-32 checksum in the chunk doesn't match the
    /// calculated checksum over the chunk type and data.
    ///
    /// # Fields
    ///
    /// - `expected`: The CRC-32 that should have been present
    /// - `actual`: The CRC-32 that was found in the chunk
    #[error("CRC verification failed: expected 0x{expected:08X}, actual 0x{actual:08X}")]
    CrcMismatch {
        /// The expected CRC-32 value
        expected: u32,
        /// The actual CRC-32 value found
        actual: u32,
    },

    /// Returned when attempting to interpret chunk data as a UTF-8 string
    /// but the data contains invalid UTF-8 sequences.
    #[error("Invalid UTF-8 in chunk data")]
    InvalidUtf8(#[from] std::str::Utf8Error),
}

impl Chunk {
    /// Maximum allowed size for chunk data according to PNG specification: 2³¹ - 1 bytes.
    pub const MAX_DATA_SIZE: usize = (1 << 31) - 1;

    /// Size of the length field in bytes: always 4 bytes (u32).
    pub const LENGTH_SIZE: usize = 4;

    /// Size of the chunk type field in bytes: always 4 bytes.
    pub const TYPE_SIZE: usize = 4;

    /// Size of the CRC field in bytes: always 4 bytes (u32).
    pub const CRC_SIZE: usize = 4;

    /// Creates a new PNG chunk by calculating the CRC-32 checksum automatically.
    ///
    /// # Arguments
    ///
    /// - `chunk_type`: The type of the chunk (e.g., IHDR, tEXt, etc.)
    /// - `data`: The payload data for the chunk
    ///
    /// # Returns
    ///
    /// A new `Chunk` instance with the CRC-32 calculated and stored.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let chunk_type = ChunkType::try_from(tEXt).unwrap();
    /// let data = bHello, PNG!.to_vec();
    /// let chunk = Chunk::new(chunk_type, data);
    /// ```
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let crc: u32 = Self::calculate_crc(&chunk_type, &data);
        Chunk {
            chunk_type,
            data,
            crc,
        }
    }

    /// Calculates the CRC-32 checksum for a chunk type and data.
    ///
    /// The CRC is calculated over the concatenated bytes of the chunk type
    /// and the chunk data, but NOT including the length field.
    ///
    /// # Arguments
    ///
    /// - `chunk_type`: Reference to the chunk type
    /// - `data`: Slice of data bytes
    ///
    /// # Returns
    ///
    /// The CRC-32 checksum as a u32.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let chunk_type = ChunkType::try_from(IHDR).unwrap();
    /// let crc = Chunk::calculate_crc(chunk_type, []);
    /// ```
    pub fn calculate_crc(chunk_type: &ChunkType, data: &[u8]) -> u32 {
        let mut digest = CRC.digest();
        digest.update(&chunk_type.bytes());
        digest.update(data);
        digest.finalize()
    }

    /// Returns the length of the chunk's data field.
    ///
    /// # Returns
    ///
    /// The data length as a u32 (big-endian format in the PNG file).
    pub fn length(&self) -> u32 {
        self.data.len() as u32
    }

    /// Returns a reference to the chunk's type.
    ///
    /// # Returns
    ///
    /// A reference to the `ChunkType` struct.
    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    /// Returns a reference to the chunk's data.
    ///
    /// # Returns
    ///
    /// A byte slice containing the chunk's payload data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the chunk's CRC-32 checksum.
    ///
    /// # Returns
    ///
    /// The CRC-32 value that was calculated over the chunk type and data.
    pub fn crc(&self) -> u32 {
        self.crc
    }

    /// Attempts to interpret the chunk data as a UTF-8 string.
    ///
    /// # Returns
    ///
    /// - `Ok(&str)`: The data as a valid UTF-8 string slice
    /// - `Err(ChunkError::InvalidUtf8)`: If the data contains invalid UTF-8 sequences
    ///
    /// # Example
    ///
    /// ```ignore
    /// let chunk = Chunk::new(chunk_type, bhello.to_vec());
    /// let text = chunk.data_as_str()?;
    /// ```
    pub fn data_as_str(&self) -> Result<&str> {
        Ok(str::from_utf8(&self.data)?)
    }

    /// Serializes the chunk to its wire format as defined in the PNG specification.
    ///
    /// The returned bytes are arranged as:
    /// 1. 4 bytes: Length (big-endian u32)
    /// 2. 4 bytes: Chunk type
    /// 3. N bytes: Data
    /// 4. 4 bytes: CRC-32 checksum (big-endian u32)
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the complete serialized chunk.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let chunk = Chunk::new(chunk_type, data);
    /// let serialized = chunk.as_bytes();
    /// ```
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(
            Self::LENGTH_SIZE + Self::TYPE_SIZE + Self::CRC_SIZE + self.data.len(),
        );
        bytes.extend_from_slice(&self.length().to_be_bytes());
        bytes.extend_from_slice(&self.chunk_type.bytes());
        bytes.extend_from_slice(&self.data);
        bytes.extend_from_slice(&self.crc.to_be_bytes());
        bytes
    }
}

/// Attempts to parse a PNG chunk from its raw byte representation.
///
/// This implementation validates the chunk structure, checks data size limits,
/// verifies the chunk type is valid, and most importantly, verifies the CRC-32
/// checksum to ensure data integrity.
///
/// # Arguments
///
/// - `bytes`: A byte slice containing the complete chunk data in PNG format
///
/// # Returns
///
/// - `Ok(Chunk)`: Successfully parsed and verified chunk
/// - `Err(ChunkError)`: Various error conditions:
///   - `NotEnoughBytes`: Input is too short to contain a valid chunk
///   - `TooLarge`: Data length exceeds PNG specification limit
///   - `InvalidChunkType`: Chunk type contains invalid characters
///   - `CrcMismatch`: CRC verification failed (data corruption)
///
/// # Example
///
/// ```ignore
/// let bytes = chunk.as_bytes();
/// let parsed_chunk = Chunk::try_from(bytes[..])?;
/// ```
impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        // Parse the length field (first 4 bytes, big-endian)
        let length_bytes: [u8; 4] = bytes
            .get(..Self::LENGTH_SIZE)
            .and_then(|slice| slice.try_into().ok())
            .ok_or_else(|| ChunkError::NotEnoughBytes {
                position: 0,
                required: Self::LENGTH_SIZE,
                actual: bytes.len() - 0,
            })?;
        let data_length = u32::from_be_bytes(length_bytes) as usize;

        // Validate data length against PNG specification limit
        if data_length > Self::MAX_DATA_SIZE {
            return Err(ChunkError::TooLarge { size: data_length });
        }

        // Parse the chunk type (next 4 bytes)
        let type_start = Self::LENGTH_SIZE;
        let type_bytes: [u8; 4] = bytes
            .get(type_start..type_start + Self::TYPE_SIZE)
            .and_then(|slice| slice.try_into().ok())
            .ok_or_else(|| ChunkError::NotEnoughBytes {
                position: type_start,
                required: Self::TYPE_SIZE,
                actual: bytes.len() - type_start,
            })?;
        let chunk_type = ChunkType::try_from(type_bytes)?;

        // Parse the data field (variable length)
        let data_start = type_start + Self::TYPE_SIZE;
        let data_bytes = bytes
            .get(data_start..data_start + data_length)
            .ok_or_else(|| ChunkError::NotEnoughBytes {
                position: data_start,
                required: data_length,
                actual: bytes.len() - data_start,
            })?
            .to_vec();

        // Parse the CRC field (last 4 bytes, big-endian)
        let crc_start = data_start + data_length;
        let crc_bytes: [u8; 4] = bytes
            .get(crc_start..crc_start + Self::CRC_SIZE)
            .and_then(|slice| slice.try_into().ok())
            .ok_or_else(|| ChunkError::NotEnoughBytes {
                position: crc_start,
                required: Self::CRC_SIZE,
                actual: bytes.len() - crc_start,
            })?;
        let crc = u32::from_be_bytes(crc_bytes);

        // Verify CRC-32 checksum integrity
        let expected_crc = Self::calculate_crc(&chunk_type, &data_bytes);
        if crc != expected_crc {
            return Err(ChunkError::CrcMismatch {
                expected: expected_crc,
                actual: crc,
            });
        }

        Ok(Self {
            chunk_type,
            data: data_bytes,
            crc,
        })
    }
}

/// Formats the chunk for display in a human-readable, multi-line format.
///
/// The output includes:
/// - Chunk length
/// - Chunk type (as 4-character string)
/// - Data bytes in hexadecimal format
/// - CRC value in hexadecimal format
///
/// # Example
///
/// ```ignore
/// println!({}, chunk);
/// ```
impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Chunk {{")?;
        writeln!(f, "  Length: {}", self.length())?;
        writeln!(f, "  Type: {}", self.chunk_type)?;

        write!(f, "  Data: [")?;
        for (i, &byte) in self.data.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{:02X}", byte)?;
        }
        writeln!(f, "]")?;
        writeln!(f, "  CRC: 0x{:08X}", self.crc)?;
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = String::from(chunk.data_as_str().unwrap());
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = String::from(chunk.data_as_str().unwrap());
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
