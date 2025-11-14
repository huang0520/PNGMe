use std::fmt;
use std::{fmt::Display, str::FromStr};

/// PNG chunk type as defined in the PNG specification.
/// Each byte represents a property via its 5th bit (0 = uppercase, 1 = lowercase):
/// - Byte 0: Critical (A-Z) vs Ancillary (a-z)
/// - Byte 1: Public (A-Z) vs Private (a-z)
/// - Byte 2: Reserved - must be uppercase
/// - Byte 3: Unsafe to copy (A-Z) vs Safe to copy (a-z)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkType {
    bytes: [u8; 4],
}

#[derive(Debug, thiserror::Error)]
pub enum ChunkTypeError {
    #[error("Byte at index {index} (0x{byte:02X}) is not an ASCII alphabetic character")]
    InvalidByte { byte: u8, index: usize },
    #[error("Invalid length: expected 4 bytes, got {0}")]
    InvalidLength(usize),
}

impl ChunkType {
    /// Returns the raw 4-byte chunk type.
    pub fn bytes(&self) -> [u8; 4] {
        self.bytes
    }

    /// Checks if all bytes are alphabetic and the reserved bit (byte 2) is valid.
    pub fn is_valid(&self) -> bool {
        self.bytes.iter().all(|&b| b.is_ascii_alphabetic()) && self.is_reserved_bit_valid()
    }

    /// True if the chunk is critical (first byte is uppercase).
    pub fn is_critical(&self) -> bool {
        !self.is_bit_set(0)
    }

    /// True if the chunk is public (second byte is uppercase).
    pub fn is_public(&self) -> bool {
        !self.is_bit_set(1)
    }

    /// True if the reserved bit (third byte) is valid (uppercase).
    pub fn is_reserved_bit_valid(&self) -> bool {
        !self.is_bit_set(2)
    }

    /// True if the chunk is safe to copy (fourth byte is lowercase).
    pub fn is_safe_to_copy(&self) -> bool {
        self.is_bit_set(3)
    }

    /// Checks if the 5th bit (0x20) is set in the byte at the given index.
    #[inline]
    fn is_bit_set(&self, index: usize) -> bool {
        const PROPERTY_BIT_MASK: u8 = 0b0010_0000;
        self.bytes[index] & PROPERTY_BIT_MASK != 0
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = ChunkTypeError;

    fn try_from(bytes: [u8; 4]) -> Result<Self, Self::Error> {
        for (index, &byte) in bytes.iter().enumerate() {
            if !byte.is_ascii_alphabetic() {
                return Err(ChunkTypeError::InvalidByte { byte, index });
            }
        }

        Ok(ChunkType { bytes })
    }
}

impl TryFrom<&[u8]> for ChunkType {
    type Error = ChunkTypeError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let bytes: [u8; 4] = slice
            .try_into()
            .map_err(|_| ChunkTypeError::InvalidLength(slice.len()))?;
        ChunkType::try_from(bytes)
    }
}

impl FromStr for ChunkType {
    type Err = ChunkTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; 4] = s
            .as_bytes()
            .try_into()
            .map_err(|_| ChunkTypeError::InvalidLength(s.len()))?;
        ChunkType::try_from(bytes)
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Safe because we validated bytes are ASCII alphabetic in try_from
        let s = str::from_utf8(&self.bytes).expect("bytes are valid ASCII");
        write!(f, "{}", s)
    }
}

impl AsRef<[u8]> for ChunkType {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
