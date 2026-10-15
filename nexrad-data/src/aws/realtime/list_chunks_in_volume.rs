use crate::aws::realtime::{ChunkIdentifier, VolumeIndex, REALTIME_BUCKET};
use crate::aws::s3::list_all_objects;

/// Lists the chunks of the newest scan generation in the specified radar site's volume, sorted by
/// sequence.
///
/// Volume numbers are reused after 999 while old chunks remain, so every listing page is read and
/// only the chunks sharing the newest date-time prefix are returned. An empty result means the
/// volume holds no chunks.
pub async fn list_chunks_in_volume(
    site: &str,
    volume: VolumeIndex,
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

            ChunkIdentifier::from_name(site, volume, identifier, object.last_modified)
        })
        .collect::<crate::result::Result<Vec<_>>>()?;

    Ok(latest_generation(metas))
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
        ChunkIdentifier::from_name("KBYX", VolumeIndex::new(589).unwrap(), name, None).unwrap()
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
    #[test]
    fn listing_is_not_capped() {
        let names: Vec<_> = (1..=150)
            .map(|sequence| format!("20260912-140452-{sequence:03}-I"))
            .collect();
        let chunks = latest_generation(names.iter().rev().map(|name| id(name)).collect());
        assert_eq!(chunks.len(), 150);
        assert_eq!(chunks.last().map(ChunkIdentifier::sequence), Some(150));
    }
}
