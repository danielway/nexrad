use crate::aws::realtime::{Chunk, ChunkIdentifier, DownloadedChunk, REALTIME_BUCKET};
use crate::aws::s3::download_object;

/// Downloads the specified chunk from the real-time NEXRAD data bucket.
///
/// The returned [`DownloadedChunk`] carries a copy of `chunk_id` with its
/// [upload time](ChunkIdentifier::upload_date_time) filled in from the object's metadata, the
/// chunk itself, and `attempts: 1`, since this function makes a single attempt and does not
/// retry.
pub async fn download_chunk(
    site: &str,
    chunk_id: &ChunkIdentifier,
) -> crate::result::Result<DownloadedChunk> {
    let key = format!(
        "{}/{}/{}",
        site,
        chunk_id.volume().as_number(),
        chunk_id.name()
    );

    let downloaded_object = download_object(REALTIME_BUCKET, &key).await?;

    Ok(DownloadedChunk {
        identifier: ChunkIdentifier::from_name(
            site.to_string(),
            *chunk_id.volume(),
            chunk_id.name().to_string(),
            downloaded_object.metadata.last_modified,
        )?,
        chunk: Chunk::new(downloaded_object.data)?,
        attempts: 1,
    })
}
