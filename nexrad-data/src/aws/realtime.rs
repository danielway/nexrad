//!
//! # Real-time NEXRAD Data
//! Near real-time (within seconds) NEXRAD radar data is uploaded to an AWS S3 bucket by NOAA. This
//! data is organized into a series of "volumes", each containing ~55 "chunks". This module provides
//! functions for identifying the most recent volume with data for a specified radar site and
//! downloading the chunks within that volume.
//!
//! A fixed number (999) volumes exist in the S3 bucket which are rotated through in a round-robin
//! fashion. Chunks are added to each volume approximately every 3 seconds with very little latency
//! from the data's collection time. TODO: quantify this using the message timestamps.
//!
//! There may be gaps in the volume data, as illustrated in the real example below from KDMX:
//! ```text
//! Volume 001: 2024-08-04 10:10:07 UTC
//! ...
//! Volume 085: 2024-08-04 17:10:49 UTC
//! Volume 086: No files found.
//! ...
//! Volume 670: No files found.
//! Volume 671: 2024-08-03 00:00:21 UTC
//! ...
//! Volume 999: 2024-08-04 10:06:37 UTC
//! ```
//! The [get_latest_volume] function will find the volume with the most recent data using a binary
//! search approach to minimize the number of network calls made. Once the latest volume is found
//! for a session, a different routine should be used to poll new data for that volume and advance
//! to the next volume when the active one is filled.
//!

use crate::aws::s3::{download_object, list_objects};
use crate::aws::search::binary_search_greatest;
use crate::result::aws::AWSError::UnrecognizedChunkFormat;
use crate::result::Error::AWS;
use crate::result::Result;
use crate::volume;
use chrono::{DateTime, Utc};

const REALTIME_BUCKET: &str = "unidata-nexrad-level2-chunks";

/// A volume's index in the AWS real-time NEXRAD bucket. These indexes are rotated-through as chunks
/// are accumulated and finally combined into full volumes to be archived.
#[derive(Debug, Copy, Clone)]
pub struct VolumeIndex(usize);

impl VolumeIndex {
    /// Creates a new volume index with the specified value.
    pub fn new(index: usize) -> Self {
        debug_assert!(index <= 999, "Volume index must be <= 999");
        Self(index)
    }

    /// Returns the volume index as a number.
    pub fn as_number(&self) -> usize {
        self.0
    }
}

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

/// The position of this chunk within the volume.
pub enum ChunkType {
    Start,
    Intermediate,
    End,
}

/// A chunk of real-time data within a volume. Chunks are ordered and when concatenated together
/// form a complete volume of radar data. All chunks contain an LDM record with radar data messages.
pub enum Chunk<'a> {
    /// The start of a new volume. This chunk will begin with an Archive II volume header followed
    /// by a compressed LDM record.
    Start(volume::File),
    /// An intermediate or end chunk. This chunk will contain a compressed LDM record with radar
    /// data messages.
    IntermediateOrEnd(volume::Record<'a>),
}

impl Chunk<'_> {
    /// Creates a new chunk from the provided data. The data is expected to be in one of two formats:
    ///
    /// 1. An Archive II volume header followed by a compressed LDM record, or a "start" chunk.
    /// 2. A compressed LDM record, or an "intermediate" or "end" chunk.
    ///
    /// The chunk type is determined by the data's format.
    pub fn new(data: Vec<u8>) -> Result<Self> {
        // Check if the data begins with an Archive II volume header, indicating a "start" chunk
        if data[0..3].as_ref() == b"AR2" {
            let file = volume::File::new(data);
            return Ok(Self::Start(file));
        }

        // Check if the data begins with a BZ compressed record, indicating an "intermediate" or "end" chunk
        if data[4..6].as_ref() == b"BZ" {
            let record = volume::Record::new(data);
            return Ok(Self::IntermediateOrEnd(record));
        }

        Err(AWS(UnrecognizedChunkFormat))
    }

    /// The data contained within this chunk.
    pub fn data(&self) -> &[u8] {
        match self {
            Self::Start(file) => file.data(),
            Self::IntermediateOrEnd(record) => record.data(),
        }
    }
}

/// Identifies the volume index with the most recent data for the specified radar site. Real-time
/// NEXRAD data is uploaded to a series of rotating volumes 0..=999, each containing ~55 chunks.
/// This function performs a binary search to find the most recent volume with data.
pub async fn get_latest_volume(site: &str) -> Result<Option<VolumeIndex>> {
    binary_search_greatest(1, 999, |volume| async move {
        let chunks = list_chunks(site, VolumeIndex::new(volume), 1).await?;
        Ok(chunks.first().map(|chunk| chunk.date_time()))
    })
    .await
    .map(|volume| volume.map(VolumeIndex::new))
}

/// Lists the chunks for the specified radar site and volume. The `max_keys` parameter can be used
/// to limit the number of chunks returned.
pub async fn list_chunks(
    site: &str,
    volume: VolumeIndex,
    max_keys: usize,
) -> Result<Vec<ChunkIdentifier>> {
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
                object.last_modified.clone(),
            )
        })
        .collect();

    Ok(metas)
}

/// Downloads the specified chunk from the real-time NEXRAD data bucket.
pub async fn download_chunk<'a>(site: &str, chunk: &ChunkIdentifier) -> Result<Chunk<'a>> {
    let key = format!("{}/{}/{}", site, chunk.volume().as_number(), chunk.name());
    let data = download_object(REALTIME_BUCKET, &key).await?;
    Chunk::new(data)
}
