use crate::aws::realtime::{ChunkIdentifier, VolumeIndex, REALTIME_BUCKET};
use crate::aws::s3::list_all_objects;
use chrono::{DateTime, Utc};
use log::warn;

/// Lists the chunks of the newest scan generation in the specified radar site's volume, sorted by
/// sequence.
///
/// Volume numbers are reused after 999 while old chunks remain, so every listing page is read and
/// only the chunks sharing the newest date-time prefix are returned. An empty result means the
/// volume holds no chunks.
///
/// Object keys whose names do not parse as a chunk identifier (for example a chunk type other
/// than `S`, `I` or `E`) are skipped with a `log` warning rather than failing the listing.
///
/// # Errors
///
/// Returns an [`AwsError`](crate::result::AwsError) if a listing request fails or its response
/// cannot be parsed.
pub async fn list_chunks_in_volume(
    site: &str,
    volume: VolumeIndex,
) -> crate::result::Result<Vec<ChunkIdentifier>> {
    let prefix = format!("{}/{}/", site, volume.number());
    let objects = list_all_objects(REALTIME_BUCKET, &prefix).await?;

    let metas = objects
        .iter()
        .filter_map(|object| parse_chunk_key(site, volume, &object.key, object.last_modified))
        .collect();

    Ok(latest_generation(metas))
}

/// Parses a listed object key into a chunk identifier, or warns and returns `None` if its name is
/// not a chunk name.
fn parse_chunk_key(
    site: &str,
    volume: VolumeIndex,
    key: &str,
    last_modified: Option<DateTime<Utc>>,
) -> Option<ChunkIdentifier> {
    let name = key.rsplit('/').next().unwrap_or(key);
    match ChunkIdentifier::from_name(site, volume, name, last_modified) {
        Ok(identifier) => Some(identifier),
        Err(error) => {
            warn!("Skipping real-time object {key:?} whose name is not a chunk name: {error}");
            None
        }
    }
}

// Select one scan generation before callers inspect chunk sequence numbers.
fn latest_generation(mut chunks: Vec<ChunkIdentifier>) -> Vec<ChunkIdentifier> {
    if let Some(newest) = chunks.iter().map(|id| id.date_time_prefix()).max() {
        chunks.retain(|id| id.date_time_prefix() == newest);
        chunks.sort_by_key(ChunkIdentifier::sequence);
    }
    chunks
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    use crate::aws::realtime::ChunkType;
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
    fn unparseable_keys_are_skipped() {
        let volume = VolumeIndex::new(589).unwrap();
        let keys = [
            "KBYX/589/20260912-140452-001-S",
            "KBYX/589/20260912-140452-002-M",
            "KBYX/589/20260912-140452-003-I",
            "KBYX/589/not-a-chunk",
            "KBYX/589/20260912-140452-xyz-I",
            "KBYX/589/20260912-140452-004-E",
        ];
        let chunks: Vec<_> = keys
            .iter()
            .filter_map(|key| parse_chunk_key("KBYX", volume, key, None))
            .collect();
        let chunks = latest_generation(chunks);
        assert_eq!(
            chunks.iter().map(|c| c.name()).collect::<Vec<_>>(),
            vec![
                "20260912-140452-001-S",
                "20260912-140452-003-I",
                "20260912-140452-004-E"
            ]
        );
        assert_eq!(
            chunks
                .iter()
                .map(ChunkIdentifier::chunk_type)
                .collect::<Vec<_>>(),
            vec![ChunkType::Start, ChunkType::Intermediate, ChunkType::End]
        );
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
