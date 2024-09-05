use crate::aws::realtime::{ChunkIdentifier, VolumeIndex, REALTIME_BUCKET};
use crate::aws::s3::list_objects;

/// Lists the chunks for the specified radar site and volume. The `max_keys` parameter can be used
/// to limit the number of chunks returned.
pub async fn list_chunks_in_volume(
    site: &str,
    volume: VolumeIndex,
    max_keys: usize,
) -> crate::result::Result<Vec<ChunkIdentifier>> {
    let prefix = format!("{}/{}/", site, volume.as_number());
    let list_result = list_objects(REALTIME_BUCKET, &prefix, Some(max_keys)).await?;

    let metas = list_result
        .objects
        .iter()
        .map(|object| {
            let identifier_segment = object.key.split('/').last();
            let identifier = identifier_segment
                .unwrap_or_else(|| object.key.as_ref())
                .to_string();

            ChunkIdentifier::new(
                site.to_string(),
                volume,
                identifier,
                Some(object.last_modified.clone()),
            )
        })
        .collect();

    Ok(metas)
}
