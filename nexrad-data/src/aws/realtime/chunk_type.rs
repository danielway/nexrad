use crate::result::{AwsError, Error, Result};

/// The position of this chunk within the volume.
///
/// Each chunk in a real-time volume has a type indicating its position
/// in the sequence of chunks that form a complete volume scan.
///
/// The Archive II ICD (2620010J §7.3.6) defines a fourth LDM key type, `M` (Model Data), which
/// does not appear in the real-time chunks bucket today and is not yet represented here. The enum
/// is non-exhaustive so it can gain that variant, or others, in a minor release.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ChunkType {
    /// The first chunk in a volume.
    Start,
    /// A chunk in the middle of a volume.
    Intermediate,
    /// The final chunk in a volume.
    End,
}

impl ChunkType {
    /// Creates a new chunk type from an abbreviation.
    ///
    /// # Errors
    ///
    /// Returns [`AwsError::UnrecognizedChunkType`](crate::result::AwsError::UnrecognizedChunkType)
    /// if `c` is not `S`, `I` or `E`.
    pub fn from_abbreviation(c: char) -> Result<Self> {
        match c {
            'S' => Ok(ChunkType::Start),
            'I' => Ok(ChunkType::Intermediate),
            'E' => Ok(ChunkType::End),
            _ => Err(Error::Aws(AwsError::UnrecognizedChunkType(Some(c)))),
        }
    }

    /// Returns the abbreviation for this chunk type.
    pub fn abbreviation(&self) -> char {
        match self {
            ChunkType::Start => 'S',
            ChunkType::Intermediate => 'I',
            ChunkType::End => 'E',
        }
    }
}
