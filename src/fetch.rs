use chrono::NaiveDate;

use crate::chunk::{ChunkMetadata, EncodedChunkFile};
use crate::result::Result;

pub async fn list_chunks(_site: &str, _date: NaiveDate) -> Result<Vec<ChunkMetadata>> {
    todo!()
}

pub async fn fetch_chunk(_metadata: &ChunkMetadata) -> Result<EncodedChunkFile> {
    todo!()
}