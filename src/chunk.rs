use chrono::NaiveDate;

#[derive(Clone, Debug)]
pub struct ChunkMeta {
    site: String,
    date: NaiveDate,
    identifier: String,
}

impl ChunkMeta {
    pub fn new(site: String, date: NaiveDate, identifier: String) -> Self {
        Self { site, date, identifier }
    }

    pub fn site(&self) -> &String {
        &self.site
    }

    pub fn date(&self) -> &NaiveDate {
        &self.date
    }

    pub fn identifier(&self) -> &String {
        &self.identifier
    }
}

pub struct EncodedChunk {
    meta: ChunkMeta,
    data: Vec<u8>,
}

impl EncodedChunk {
    pub fn new(meta: ChunkMeta, data: Vec<u8>) -> Self {
        Self { meta, data }
    }

    pub fn meta(&self) -> &ChunkMeta {
        &self.meta
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn compressed(&self) -> Option<bool> {
        todo!()
    }
}

pub struct Chunk {
    meta: ChunkMeta,
    // TODO
}

impl Chunk {
    pub fn new(meta: ChunkMeta) -> Self {
        Self { meta }
    }

    pub fn meta(&self) -> &ChunkMeta {
        &self.meta
    }
}