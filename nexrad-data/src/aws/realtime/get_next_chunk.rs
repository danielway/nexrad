use crate::aws::realtime::{ChunkIdentifier, ChunkType, VolumeIndex};

/// Determines which chunk is next in the sequence after the specified chunk. If the chunk is the
/// last in the volume, the next chunk will be the first in the next volume.
pub fn get_next_chunk(site: &str, chunk: ChunkIdentifier) -> Option<ChunkIdentifier> {
    let mut sequence = chunk.sequence()?;
    let mut volume = *chunk.volume();
    let mut chunk_type;

    if sequence == 55 {
        sequence = 1;
        chunk_type = ChunkType::Start;

        if volume == VolumeIndex::new(999) {
            volume = VolumeIndex::new(1);
        } else {
            volume = VolumeIndex::new(volume.as_number() + 1);
        }
    } else {
        sequence += 1;
        chunk_type = ChunkType::Intermediate;

        if sequence == 55 {
            chunk_type = ChunkType::End;
        }
    }

    let name = format!(
        "{}-{:03}-{}",
        chunk.name_prefix(),
        sequence,
        match chunk_type {
            ChunkType::Start => "S",
            ChunkType::Intermediate => "I",
            ChunkType::End => "E",
        }
    );

    Some(ChunkIdentifier::new(
        site.to_string(),
        volume,
        name,
        chunk.date_time(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aws::realtime::VolumeIndex;
    use chrono::{TimeZone, Utc};

    #[tokio::test]
    async fn test_get_next_chunk_guess_start() {
        let site = "KTLX";
        let volume = VolumeIndex::new(50);
        let name = "20240813-123330-001-S";
        let date_time = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();

        let chunk = ChunkIdentifier::new(site.to_string(), volume, name.to_string(), date_time);

        let next_chunk = get_next_chunk(site, chunk).unwrap();
        assert_eq!(next_chunk.site(), site);
        assert_eq!(next_chunk.volume().as_number(), 50);
        assert_eq!(next_chunk.name(), "20240813-123330-002-I");
        assert_eq!(next_chunk.name_prefix(), "20240813-123330");
        assert_eq!(next_chunk.sequence(), Some(2));
        assert_eq!(next_chunk.chunk_type(), Some(ChunkType::Intermediate));
        assert_eq!(next_chunk.date_time(), date_time);
    }

    #[tokio::test]
    async fn test_get_next_chunk_guess_intermediate() {
        let site = "KTLX";
        let volume = VolumeIndex::new(50);
        let name = "20240813-123330-014-I";
        let date_time = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();

        let chunk = ChunkIdentifier::new(site.to_string(), volume, name.to_string(), date_time);

        let next_chunk = get_next_chunk(site, chunk).unwrap();
        assert_eq!(next_chunk.site(), site);
        assert_eq!(next_chunk.volume().as_number(), 50);
        assert_eq!(next_chunk.name(), "20240813-123330-015-I");
        assert_eq!(next_chunk.name_prefix(), "20240813-123330");
        assert_eq!(next_chunk.sequence(), Some(15));
        assert_eq!(next_chunk.chunk_type(), Some(ChunkType::Intermediate));
        assert_eq!(next_chunk.date_time(), date_time);
    }

    #[tokio::test]
    async fn test_get_next_chunk_guess_end() {
        let site = "KTLX";
        let volume = VolumeIndex::new(50);
        let name = "20240813-123330-055-E";
        let date_time = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();

        let chunk = ChunkIdentifier::new(site.to_string(), volume, name.to_string(), date_time);

        let next_chunk = get_next_chunk(site, chunk).unwrap();
        assert_eq!(next_chunk.site(), site);
        assert_eq!(next_chunk.volume().as_number(), 51);
        assert_eq!(next_chunk.name(), "20240813-123330-001-S");
        assert_eq!(next_chunk.name_prefix(), "20240813-123330");
        assert_eq!(next_chunk.sequence(), Some(1));
        assert_eq!(next_chunk.chunk_type(), Some(ChunkType::Start));
        assert_eq!(next_chunk.date_time(), date_time);
    }

    #[tokio::test]
    async fn test_get_next_chunk_guess_last_volume() {
        let site = "KTLX";
        let volume = VolumeIndex::new(999);
        let name = "20240813-123330-055-E";
        let date_time = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();

        let chunk = ChunkIdentifier::new(site.to_string(), volume, name.to_string(), date_time);

        let next_chunk = get_next_chunk(site, chunk).unwrap();
        assert_eq!(next_chunk.site(), site);
        assert_eq!(next_chunk.volume().as_number(), 1);
        assert_eq!(next_chunk.name(), "20240813-123330-001-S");
        assert_eq!(next_chunk.name_prefix(), "20240813-123330");
        assert_eq!(next_chunk.sequence(), Some(1));
        assert_eq!(next_chunk.chunk_type(), Some(ChunkType::Start));
        assert_eq!(next_chunk.date_time(), date_time);
    }
}
