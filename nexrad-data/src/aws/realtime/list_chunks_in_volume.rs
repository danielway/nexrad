use crate::aws::realtime::{ChunkIdentifier, VolumeIndex, REALTIME_BUCKET};
use crate::aws::s3::list_all_objects;

/// Lists only the newest scan generation for the specified radar site and volume.
/// Read all pages before limiting results: volume numbers are reused while old chunks remain. The `max_keys` parameter can be used
/// to limit the number of chunks returned.
pub async fn list_chunks_in_volume(
    site: &str,
    volume: VolumeIndex,
    max_keys: usize,
) -> crate::result::Result<Vec<ChunkIdentifier>> {
    let prefix = format!("{}/{}/", site, volume.as_number());
    let objects = list_all_objects(REALTIME_BUCKET, &prefix).await?;

    let metas = objects
        .iter()
        .map(|object| {
            let identifier_segment = object.key.split('/').next_back();
            let identifier = identifier_segment
                .unwrap_or_else(|| object.key.as_ref())
                .to_string();

            ChunkIdentifier::from_name(site.to_string(), volume, identifier, object.last_modified)
        })
        .collect::<crate::result::Result<Vec<_>>>()?;

    let mut latest = latest_generation(metas);
    latest.truncate(max_keys);
    Ok(latest)
}

// Select one scan generation before callers inspect chunk sequence numbers.
fn latest_generation(mut chunks: Vec<ChunkIdentifier>) -> Vec<ChunkIdentifier> {
    if let Some(newest) = chunks.iter().map(|id| *id.date_time_prefix()).max() {
        chunks.retain(|id| *id.date_time_prefix() == newest);
        chunks.sort_by_key(ChunkIdentifier::sequence);
    }
    chunks
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    fn id(name: &str) -> ChunkIdentifier {
        ChunkIdentifier::from_name("KBYX".into(), VolumeIndex::new(589), name.into(), None).unwrap()
    }
    #[test]
    fn reused_volume_contains_only_newest_scan_in_sequence_order() {
        let chunks = latest_generation(vec![
            id("20260909-013029-001-S"),
            id("20260912-140452-002-I"),
            id("20260909-013029-052-E"),
            id("20260912-140452-001-S"),
        ]);
        assert_eq!(
            chunks.iter().map(|c| c.name()).collect::<Vec<_>>(),
            vec!["20260912-140452-001-S", "20260912-140452-002-I"]
        );
    }
    #[test]
    fn expired_start_does_not_make_old_generation_usable() {
        let chunks = latest_generation(vec![
            id("20260909-013029-001-S"),
            id("20260912-140452-003-I"),
        ]);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].sequence(), 3);
    }
    #[test]
    fn empty_volume_stays_empty() {
        assert!(latest_generation(vec![]).is_empty());
    }
}
