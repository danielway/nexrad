use crate::aws::realtime::{Chunk, ChunkIdentifier, REALTIME_BUCKET};
use crate::aws::s3::download_object;

/// Downloads the specified chunk from the real-time NEXRAD data bucket.
pub async fn download_chunk<'a>(
    site: &str,
    chunk: &ChunkIdentifier,
) -> crate::result::Result<Chunk<'a>> {
    let key = format!("{}/{}/{}", site, chunk.volume().as_number(), chunk.name());
    let data = download_object(REALTIME_BUCKET, &key).await?;
    Chunk::new(data)
}
