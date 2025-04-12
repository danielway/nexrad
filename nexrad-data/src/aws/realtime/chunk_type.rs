use crate::result::{aws::AWSError, Error, Result};

/// The position of this chunk within the volume.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ChunkType {
    Start,
    Intermediate,
    End,
}

impl ChunkType {
    /// Creates a new chunk type from an abbreviation.
    pub fn from_abbreviation(c: char) -> Result<Self> {
        match c {
            'S' => Ok(ChunkType::Start),
            'I' => Ok(ChunkType::Intermediate),
            'E' => Ok(ChunkType::End),
            _ => Err(Error::AWS(AWSError::UnrecognizedChunkType(Some(c)))),
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
