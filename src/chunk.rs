use std::fmt;

use crate::{chunk_type::ChunkType, chunk_type::ChunkTypeError};
use crc::{CRC_32_ISO_HDLC, Crc};

/// CRC-32 algorithm used for PNG chunk verification (ISO/HDLC).
const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
// LENGTH(4) + CHUNK_TYPE(4)
const HEADER_SIZE: usize = 8;
const CRC_SIZE: usize = 4;

/// PNG chunk as defined in the PNG specification.
///
/// # Format
/// - Length: 4 bytes (big-endian u32)
/// - Chunk Type: 4 bytes
/// - Data: variable length
/// - CRC: 4 bytes (over chunk type + data)
pub struct Chunk {
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

/// Errors that can occur when parsing or constructing a PNG chunk.
#[derive(Debug, thiserror::Error)]
pub enum ChunkError {
    #[error("Insufficient data: need at least {required} bytes, got {actual}")]
    NotEnoughBytes { required: usize, actual: usize },

    #[error("Invalid chunk type")]
    InvalidChunkType(#[from] ChunkTypeError),

    #[error("CRC verification failed: expected 0x{expected:08X}, actual 0x{actual:08X}")]
    CrcMismatch { expected: u32, actual: u32 },
}

impl Chunk {
    /// Creates a new chunk by calculating CRC from type and data.
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let crc: u32 = Self::calculate_crc(&chunk_type, &data);
        Chunk {
            chunk_type,
            data,
            crc,
        }
    }

    /// Creates a chunk from raw components without CRC calculation.
    /// Only used internally for TryFrom after CRC verification.
    fn from_verified_parts(chunk_type: ChunkType, data: Vec<u8>, crc: u32) -> Self {
        Chunk {
            chunk_type,
            data,
            crc,
        }
    }

    /// Calculates the CRC-32 checksum for a chunk type and data.
    fn calculate_crc(chunk_type: &ChunkType, data: &[u8]) -> u32 {
        let mut digest = CRC.digest();
        digest.update(&chunk_type.bytes());
        digest.update(data);
        digest.finalize()
    }

    fn length(&self) -> u32 {
        self.data.len() as u32
    }

    fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn crc(&self) -> u32 {
        self.crc
    }

    /// Returns data as a UTF-8 string slice.
    pub fn data_as_str(&self) -> Result<&str, std::str::Utf8Error> {
        str::from_utf8(&self.data)
    }

    /// Returns data as an owned UTF-8 String.
    pub fn data_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.data.clone())
    }

    /// Serializes the chunk to its wire format.
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_SIZE + CRC_SIZE + self.data.len());
        bytes.extend_from_slice(&self.length().to_be_bytes());
        bytes.extend_from_slice(&self.chunk_type.bytes());
        bytes.extend_from_slice(&self.data);
        bytes.extend_from_slice(&self.crc.to_be_bytes());
        bytes
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        const MIN_SIZE: usize = HEADER_SIZE + CRC_SIZE;

        if bytes.len() < MIN_SIZE {
            return Err(ChunkError::NotEnoughBytes {
                required: MIN_SIZE,
                actual: bytes.len(),
            });
        }

        // Check declared data lenght match actual data length
        let declared_length = u32::from_be_bytes(
            bytes[0..4]
                .try_into()
                .expect("MIN_SIZE check ensures length bytes is valid"),
        );
        let data_len = declared_length as usize;
        let total_len = HEADER_SIZE + data_len + CRC_SIZE;

        if bytes.len() < total_len {
            return Err(ChunkError::NotEnoughBytes {
                required: total_len,
                actual: bytes.len(),
            });
        }

        let chunk_bytes: [u8; 4] = bytes[4..8]
            .try_into()
            .expect("MIN_SIZE check ensures chunk_type bytes is valid");
        let chunk_type = ChunkType::try_from(chunk_bytes)?;

        let data = bytes[8..8 + data_len].to_vec();

        let crc_start = 8 + data_len;
        let crc_bytes: [u8; 4] = bytes[crc_start..crc_start + CRC_SIZE]
            .try_into()
            .expect("total_len check ensures crc bytes is valid");
        let crc = u32::from_be_bytes(crc_bytes);

        // Verify CRC
        let expected_crc = Self::calculate_crc(&chunk_type, &data);
        if crc != expected_crc {
            return Err(ChunkError::CrcMismatch {
                expected: expected_crc,
                actual: crc,
            });
        }

        Ok(Self::from_verified_parts(chunk_type, data, crc))
    }
}

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

impl AsRef<[u8]> for Chunk {
    fn as_ref(&self) -> &[u8] {
        &self.data
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
        let chunk_string = chunk.data_as_string().unwrap();
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

        let chunk_string = chunk.data_as_string().unwrap();
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
