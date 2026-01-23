//! Pull-based chunk iterator for real-time NEXRAD data.
//!
//! This module provides a pull-based iterator that allows callers to control
//! timing externally, making it suitable for environments without tokio or
//! where manual timing control is preferred.

use crate::aws::realtime::{
    download_chunk, estimate_chunk_availability_time, get_latest_volume, list_chunks_in_volume,
    Chunk, ChunkIdentifier, ChunkTimingStats, ChunkType, ElevationChunkMapper, NextChunk,
    RetryPolicy, RetryState, VolumeIndex,
};
use crate::result::{aws::AWSError, Error, Result};
use chrono::{DateTime, Duration, Utc};
use log::debug;
use nexrad_decode::messages::volume_coverage_pattern;

/// A downloaded chunk with metadata about the download process.
#[derive(Debug)]
pub struct DownloadedChunk {
    /// The chunk identifier.
    pub identifier: ChunkIdentifier,
    /// The chunk data.
    pub chunk: Chunk<'static>,
    /// Number of attempts required to download this chunk.
    pub attempts: usize,
}

/// Iterator state for tracking what to fetch next.
#[derive(Debug, Clone, PartialEq, Eq)]
enum IteratorState {
    /// Need to fetch the start chunk for a new volume.
    NeedVolumeStart(VolumeIndex),
    /// Ready to fetch the next chunk in sequence.
    Ready(ChunkIdentifier),
}

/// Pull-based iterator for real-time NEXRAD chunks.
///
/// This iterator allows manual control over timing, making it suitable for
/// environments without tokio or where caller-controlled scheduling is preferred.
/// Instead of blocking, callers can use [`next_expected_time`](Self::next_expected_time)
/// to determine when to call [`try_next`](Self::try_next).
pub struct ChunkIterator {
    site: String,
    state: IteratorState,
    elevation_mapper: Option<ElevationChunkMapper>,
    vcp: Option<volume_coverage_pattern::Message<'static>>,
    timing_stats: ChunkTimingStats,
    download_policy: RetryPolicy,
    discovery_policy: RetryPolicy,
    last_chunk_time: Option<DateTime<Utc>>,
}

impl ChunkIterator {
    /// Creates a new chunk iterator starting at the latest available volume.
    ///
    /// This will make network requests to discover the latest volume and its
    /// most recent chunk.
    pub async fn new(site: &str) -> Result<Self> {
        Self::with_policies(
            site,
            RetryPolicy::default_download(),
            RetryPolicy::default_discovery(),
        )
        .await
    }

    /// Creates a new chunk iterator with custom retry policies.
    pub async fn with_policies(
        site: &str,
        download_policy: RetryPolicy,
        discovery_policy: RetryPolicy,
    ) -> Result<Self> {
        let latest_volume_result = get_latest_volume(site).await?;
        let volume = latest_volume_result
            .volume
            .ok_or(AWSError::LatestVolumeNotFound)?;

        debug!(
            "ChunkIterator initialized for site {} at volume {}",
            site,
            volume.as_number()
        );

        Ok(Self {
            site: site.to_string(),
            state: IteratorState::NeedVolumeStart(volume),
            elevation_mapper: None,
            vcp: None,
            timing_stats: ChunkTimingStats::new(),
            download_policy,
            discovery_policy,
            last_chunk_time: None,
        })
    }

    /// Creates a new chunk iterator starting at a specific chunk.
    ///
    /// This is useful for resuming from a known position.
    pub fn from_chunk(
        site: &str,
        chunk_id: ChunkIdentifier,
        download_policy: RetryPolicy,
        discovery_policy: RetryPolicy,
    ) -> Self {
        Self {
            site: site.to_string(),
            state: IteratorState::Ready(chunk_id),
            elevation_mapper: None,
            vcp: None,
            timing_stats: ChunkTimingStats::new(),
            download_policy,
            discovery_policy,
            last_chunk_time: None,
        }
    }

    /// Returns the estimated time when the next chunk will be available.
    ///
    /// Returns `None` if timing cannot be estimated (e.g., VCP not yet known).
    /// The caller can use this to schedule when to call [`try_next`](Self::try_next).
    pub fn next_expected_time(&self) -> Option<DateTime<Utc>> {
        let chunk_id = match &self.state {
            IteratorState::NeedVolumeStart(_) => return None,
            IteratorState::Ready(id) => id,
        };

        let vcp = self.vcp.as_ref()?;
        let mapper = self.elevation_mapper.as_ref()?;

        estimate_chunk_availability_time(chunk_id, vcp, mapper, Some(&self.timing_stats))
    }

    /// Returns the estimated duration until the next chunk is available.
    ///
    /// Returns `None` if timing cannot be estimated or if the chunk should
    /// already be available.
    pub fn time_until_next(&self) -> Option<Duration> {
        let expected = self.next_expected_time()?;
        let now = Utc::now();

        if expected <= now {
            None
        } else {
            Some(expected - now)
        }
    }

    /// Attempts to fetch the next chunk.
    ///
    /// Returns:
    /// - `Ok(Some(chunk))` if a chunk was successfully downloaded
    /// - `Ok(None)` if the chunk is not yet available (caller should wait and retry)
    /// - `Err(...)` if an unrecoverable error occurred
    ///
    /// This method uses the configured retry policy for transient failures.
    pub async fn try_next(&mut self) -> Result<Option<DownloadedChunk>> {
        match &self.state {
            IteratorState::NeedVolumeStart(volume) => {
                self.try_fetch_volume_start(*volume).await
            }
            IteratorState::Ready(current) => {
                let next = current
                    .next_chunk(self.elevation_mapper.as_ref().ok_or(AWSError::FailedToDetermineNextChunk)?)
                    .ok_or(AWSError::FailedToDetermineNextChunk)?;

                match next {
                    NextChunk::Sequence(next_id) => self.try_fetch_chunk(next_id).await,
                    NextChunk::Volume(next_volume) => {
                        self.try_fetch_volume_start(next_volume).await
                    }
                }
            }
        }
    }

    /// Attempts to fetch the start chunk of a new volume.
    async fn try_fetch_volume_start(&mut self, volume: VolumeIndex) -> Result<Option<DownloadedChunk>> {
        let mut retry_state = RetryState::new(self.discovery_policy.clone());

        while retry_state.should_retry() {
            match self.fetch_latest_chunk_in_volume(volume).await {
                Ok(Some(downloaded)) => {
                    // Update VCP and elevation mapper from start chunk
                    if downloaded.identifier.chunk_type() == ChunkType::Start {
                        if let Ok(vcp) = Self::extract_vcp(&downloaded.chunk) {
                            self.elevation_mapper = Some(ElevationChunkMapper::new(&vcp));
                            self.vcp = Some(vcp);
                        }
                    }

                    // Update timing stats if we have previous chunk time
                    if let (Some(upload_time), Some(prev_time)) =
                        (downloaded.identifier.upload_date_time(), self.last_chunk_time)
                    {
                        let duration = upload_time - prev_time;
                        self.update_timing_stats(&downloaded.identifier, duration, downloaded.attempts);
                    }

                    self.last_chunk_time = downloaded.identifier.upload_date_time();
                    self.state = IteratorState::Ready(downloaded.identifier.clone());

                    return Ok(Some(downloaded));
                }
                Ok(None) => {
                    // Volume has no chunks yet, will retry
                }
                Err(e) => {
                    debug!("Error fetching volume start: {:?}", e);
                }
            }

            if let Some(_delay) = retry_state.next_delay() {
                // Return None to let the caller control timing.
                // The caller should check time_until_next() and schedule accordingly.
                return Ok(None);
            }
        }

        // Exhausted retries
        Err(Error::AWS(AWSError::ExpectedChunkNotFound))
    }

    /// Fetches the latest chunk in a volume.
    async fn fetch_latest_chunk_in_volume(
        &self,
        volume: VolumeIndex,
    ) -> Result<Option<DownloadedChunk>> {
        let chunks = list_chunks_in_volume(&self.site, volume, 100).await?;
        let latest = match chunks.last() {
            Some(id) => id,
            None => return Ok(None),
        };

        let (identifier, chunk) = download_chunk(&self.site, latest).await?;

        Ok(Some(DownloadedChunk {
            identifier,
            chunk,
            attempts: 1,
        }))
    }

    /// Attempts to fetch a specific chunk.
    async fn try_fetch_chunk(&mut self, chunk_id: ChunkIdentifier) -> Result<Option<DownloadedChunk>> {
        let mut retry_state = RetryState::new(self.download_policy.clone());
        let mut attempts = 0;

        while retry_state.should_retry() {
            attempts += 1;

            match download_chunk(&self.site, &chunk_id).await {
                Ok((identifier, chunk)) => {
                    // Update VCP if this is a start chunk
                    if identifier.chunk_type() == ChunkType::Start {
                        if let Ok(vcp) = Self::extract_vcp(&chunk) {
                            self.elevation_mapper = Some(ElevationChunkMapper::new(&vcp));
                            self.vcp = Some(vcp);
                        }
                    }

                    // Update timing stats
                    if let (Some(upload_time), Some(prev_time)) =
                        (identifier.upload_date_time(), self.last_chunk_time)
                    {
                        let duration = upload_time - prev_time;
                        self.update_timing_stats(&identifier, duration, attempts);
                    }

                    self.last_chunk_time = identifier.upload_date_time();
                    self.state = IteratorState::Ready(identifier.clone());

                    return Ok(Some(DownloadedChunk {
                        identifier,
                        chunk,
                        attempts,
                    }));
                }
                Err(Error::AWS(AWSError::S3ObjectNotFoundError)) => {
                    // Chunk not yet available, retry
                    debug!(
                        "Chunk {} not yet available, attempt {}",
                        chunk_id.name(),
                        attempts
                    );
                }
                Err(e) => {
                    debug!("Error downloading chunk: {:?}", e);
                    return Err(e);
                }
            }

            if retry_state.next_delay().is_none() {
                break;
            }

            // Return None to let caller control timing
            return Ok(None);
        }

        // Chunk not available after configured retries
        Ok(None)
    }

    /// Extracts VCP from a start chunk.
    fn extract_vcp(chunk: &Chunk) -> Result<volume_coverage_pattern::Message<'static>> {
        if let Chunk::Start(file) = chunk {
            for mut record in file.records()? {
                if record.compressed() {
                    record = record.decompress()?;
                }

                for message in record.messages()? {
                    if let nexrad_decode::messages::MessageContents::VolumeCoveragePattern(vcp) =
                        message.contents()
                    {
                        return Ok(vcp.clone().into_owned());
                    }
                }
            }
        }

        Err(Error::MissingCoveragePattern)
    }

    /// Updates timing statistics for a chunk.
    fn update_timing_stats(
        &mut self,
        chunk_id: &ChunkIdentifier,
        duration: Duration,
        attempts: usize,
    ) {
        if let (Some(vcp), Some(mapper)) = (&self.vcp, &self.elevation_mapper) {
            if let Some(elevation) = mapper
                .get_sequence_elevation_number(chunk_id.sequence())
                .and_then(|n| vcp.elevations().get(n - 1))
            {
                use crate::aws::realtime::ChunkCharacteristics;

                let characteristics = ChunkCharacteristics {
                    chunk_type: chunk_id.chunk_type(),
                    waveform_type: elevation.waveform_type(),
                    channel_configuration: elevation.channel_configuration(),
                };

                self.timing_stats.add_timing(characteristics, duration, attempts);
            }
        }
    }

    /// Returns the current chunk identifier, if available.
    pub fn current(&self) -> Option<&ChunkIdentifier> {
        match &self.state {
            IteratorState::Ready(id) => Some(id),
            IteratorState::NeedVolumeStart(_) => None,
        }
    }

    /// Returns a reference to the timing statistics.
    pub fn timing_stats(&self) -> &ChunkTimingStats {
        &self.timing_stats
    }

    /// Returns a mutable reference to the timing statistics.
    pub fn timing_stats_mut(&mut self) -> &mut ChunkTimingStats {
        &mut self.timing_stats
    }

    /// Returns the current VCP if available.
    pub fn vcp(&self) -> Option<&volume_coverage_pattern::Message<'static>> {
        self.vcp.as_ref()
    }

    /// Returns the current elevation chunk mapper if available.
    pub fn elevation_mapper(&self) -> Option<&ElevationChunkMapper> {
        self.elevation_mapper.as_ref()
    }

    /// Returns the site identifier.
    pub fn site(&self) -> &str {
        &self.site
    }

    /// Returns the download retry policy.
    pub fn download_policy(&self) -> &RetryPolicy {
        &self.download_policy
    }

    /// Returns the discovery retry policy.
    pub fn discovery_policy(&self) -> &RetryPolicy {
        &self.discovery_policy
    }
}
