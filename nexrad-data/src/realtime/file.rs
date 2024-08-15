use crate::realtime::Chunk;

/// Represents a chunk data file stored in the NEXRAD real-time bucket.
pub struct File {
    chunk: Chunk,
    data: Vec<u8>,
}

impl File {
    pub(crate) fn new(chunk: Chunk, data: Vec<u8>) -> Self {
        Self { chunk, data }
    }

    /// The chunk metadata associated with this file.
    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    /// The raw data contents of this file.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
