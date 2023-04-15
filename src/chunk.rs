#[derive(Debug)]
pub struct ChunkMetadata {}

pub struct EncodedChunkFile {
    meta: ChunkMetadata,
    data: Vec<u8>,
}

impl EncodedChunkFile {
    pub fn new(meta: ChunkMetadata, data: Vec<u8>) -> Self {
        Self { meta, data }
    }

    pub fn meta(&self) -> &ChunkMetadata {
        &self.meta
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn compressed(&self) -> Option<bool> {
        todo!()
    }
}

pub struct DecodedChunkFile {
    meta: ChunkMetadata,
    // TODO
}

impl DecodedChunkFile {
    pub fn new(meta: ChunkMetadata) -> Self {
        Self { meta }
    }

    pub fn meta(&self) -> &ChunkMetadata {
        &self.meta
    }
}