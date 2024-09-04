use crate::aws::realtime::{ChunkType, VolumeIndex};
use chrono::{DateTime, Utc};

/// Identifies a volume chunk within the real-time NEXRAD data bucket. These chunks are uploaded
/// every few seconds and contain a portion of the radar data for a specific volume.
#[derive(Debug, Clone)]
pub struct ChunkIdentifier {
    site: String,
    volume: VolumeIndex,
    name: String,
    date_time: DateTime<Utc>,
}

impl ChunkIdentifier {
    /// Creates a new chunk identifier.
    pub fn new(site: String, volume: VolumeIndex, name: String, date_time: DateTime<Utc>) -> Self {
        Self {
            site,
            volume,
            name,
            date_time,
        }
    }

    /// The chunk's radar site identifier.
    pub fn site(&self) -> &str {
        &self.site
    }

    /// The chunk's rotating volume index.
    pub fn volume(&self) -> &VolumeIndex {
        &self.volume
    }

    /// The chunk's name.
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// The chunk's name prefix.
    pub fn name_prefix(&self) -> &str {
        &self.name[..15]
    }
    
    /// The sequence number of this chunk within the volume.
    pub fn sequence(&self) -> Option<usize> {
        self.name.split('-').nth(2).and_then(|s| s.parse().ok())
    }

    /// The position of this chunk within the volume.
    pub fn chunk_type(&self) -> Option<ChunkType> {
        match self.name.chars().last() {
            Some('S') => Some(ChunkType::Start),
            Some('I') => Some(ChunkType::Intermediate),
            Some('E') => Some(ChunkType::End),
            _ => None,
        }
    }

    /// The date and time this chunk was uploaded.
    pub fn date_time(&self) -> DateTime<Utc> {
        self.date_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use chrono::TimeZone;

    #[test]
    fn test_chunk_identifier() {
        let site = "KTLX";
        let volume = 50;
        let name = "20240813-123330-014-I";
        let date_time = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
        
        let chunk = ChunkIdentifier::new(
            site.to_string(), 
            VolumeIndex::new(volume), 
            name.to_string(),
            date_time,
        );

        assert_eq!(chunk.site(), site);
        assert_eq!(chunk.volume().as_number(), 50);
        assert_eq!(chunk.name(), name);
        assert_eq!(chunk.name_prefix(), "20240813-123330");
        assert_eq!(chunk.chunk_type(), Some(ChunkType::Intermediate));
        assert_eq!(chunk.sequence(), Some(14));
        assert_eq!(chunk.date_time(), date_time);
    }
}
