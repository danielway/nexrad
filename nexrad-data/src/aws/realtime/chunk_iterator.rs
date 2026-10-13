//! Pull-based chunk iterator for real-time NEXRAD data.
//!
//! This module provides a pull-based iterator that allows callers to control
//! timing externally, making it suitable for environments without tokio or
//! where manual timing control is preferred.

use crate::aws::realtime::{
    download_chunk, estimate_chunk_availability_time, get_latest_volume, list_chunks_in_volume,
    project_volume_end_time, Chunk, ChunkCharacteristics, ChunkIdentifier, ChunkTimingStats,
    ChunkType, ElevationChunkMapper, NextChunk, PollConfig, RetryState, VolumeIndex,
};
use crate::result::{AwsError, Error, Result};
use chrono::{DateTime, Utc};
use log::debug;
use nexrad_decode::messages::volume_coverage_pattern;

/// A downloaded chunk with metadata about the download process.
///
/// Returned by [`download_chunk`](super::download_chunk), [`ChunkIterator`], and
/// [`chunk_stream`](super::chunk_stream). The fields are public so they can be read and moved
/// out. The struct is `#[non_exhaustive]` so fields may be added in a minor release; construct it
/// with [`DownloadedChunk::new`] rather than a struct literal.
#[derive(Debug)]
#[non_exhaustive]
pub struct DownloadedChunk {
    /// The chunk identifier.
    pub identifier: ChunkIdentifier,
    /// The chunk data.
    pub chunk: Chunk,
    /// Number of attempts required to download this chunk.
    pub attempts: usize,
}

impl DownloadedChunk {
    /// Creates a downloaded chunk from its identifier, data, and the number of download attempts
    /// it took.
    pub fn new(identifier: ChunkIdentifier, chunk: Chunk, attempts: usize) -> Self {
        Self {
            identifier,
            chunk,
            attempts,
        }
    }
}

/// Result of initializing a [`ChunkIterator`].
///
/// When creating a new iterator, the latest chunk in the volume is fetched and returned
/// in `latest_chunk`. If joining mid-volume (the latest chunk is not a Start chunk),
/// the Start chunk is also fetched to extract the VCP and is returned in `start_chunk`.
///
/// The fields are public so they can be read and moved out, but the struct is
/// `#[non_exhaustive]`: it cannot be constructed outside this crate, and fields may be added in a
/// minor release.
#[derive(Debug)]
#[non_exhaustive]
pub struct ChunkIteratorInit {
    /// The initialized iterator, ready for subsequent `try_next()` calls.
    pub iterator: ChunkIterator,
    /// The latest chunk in the volume (the chunk the iterator joined at).
    pub latest_chunk: DownloadedChunk,
    /// The Start chunk, if it was fetched separately from the latest chunk.
    /// This is `Some` when joining mid-volume and contains the VCP metadata.
    /// When the latest chunk IS the Start chunk, this is `None`.
    pub start_chunk: Option<DownloadedChunk>,
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
///
/// Within a volume, [`try_next`](Self::try_next) returns chunks in sequence order. When a volume
/// ends, it waits for the next volume and always returns its Start chunk (sequence 1, carrying the
/// volume header and VCP) first, then continues from sequence 2, even if later chunks were already
/// published while it waited. The [VCP](Self::vcp) and elevation mapping are refreshed from each
/// Start chunk, so every volume can be passed to `assemble_volume` and a VCP change at rollover is
/// picked up.
#[derive(Debug)]
pub struct ChunkIterator {
    config: PollConfig,
    state: IteratorState,
    elevation_mapper: Option<ElevationChunkMapper>,
    vcp: Option<volume_coverage_pattern::Message<'static>>,
    timing_stats: ChunkTimingStats,
    last_chunk_time: Option<DateTime<Utc>>,
    requests_made: usize,
    bytes_downloaded: u64,
}

impl ChunkIterator {
    /// Starts a new chunk iterator at the latest available volume.
    ///
    /// This will make network requests to discover the latest volume and download
    /// the most recent chunk. The returned [`ChunkIteratorInit`] contains both the
    /// iterator and the initial chunk(s), including the Start chunk if joining mid-volume.
    ///
    /// # Returns
    ///
    /// A [`ChunkIteratorInit`] containing:
    /// - `iterator`: The initialized iterator ready for `try_next()` calls
    /// - `latest_chunk`: The most recent chunk in the volume
    /// - `start_chunk`: The Start chunk if it differs from `latest_chunk` (contains VCP metadata)
    ///
    /// This is shorthand for [`start_with(PollConfig::new(site))`](Self::start_with).
    pub async fn start(site: &str) -> Result<ChunkIteratorInit> {
        Self::start_with(PollConfig::new(site)).await
    }

    /// Starts a new chunk iterator with the given configuration.
    ///
    /// See [`start`](Self::start) for details on the return value.
    pub async fn start_with(config: PollConfig) -> Result<ChunkIteratorInit> {
        let latest = get_latest_volume(config.site()).await?;
        let volume = latest.volume.ok_or(AwsError::LatestVolumeNotFound)?;

        debug!(
            "ChunkIterator initialized for site {} at volume {}",
            config.site(),
            volume.as_number()
        );

        let mut iterator = Self {
            config,
            state: IteratorState::NeedVolumeStart(volume),
            elevation_mapper: None,
            vcp: None,
            timing_stats: ChunkTimingStats::new(),
            last_chunk_time: None,
            requests_made: latest.calls,
            bytes_downloaded: 0,
        };

        // Fetch the initial chunk(s)
        let (latest_chunk, start_chunk) = iterator.fetch_initial_chunks(volume).await?;

        Ok(ChunkIteratorInit {
            iterator,
            latest_chunk,
            start_chunk,
        })
    }

    /// Creates a new chunk iterator starting at a specific chunk.
    ///
    /// This is useful for resuming from a known position. Note that the VCP and
    /// elevation mapper will not be available until a Start chunk is encountered.
    pub fn from_chunk(config: PollConfig, chunk_id: ChunkIdentifier) -> Self {
        Self {
            config,
            state: IteratorState::Ready(chunk_id),
            elevation_mapper: None,
            vcp: None,
            timing_stats: ChunkTimingStats::new(),
            last_chunk_time: None,
            requests_made: 0,
            bytes_downloaded: 0,
        }
    }

    /// Fetches the initial chunks during iterator construction.
    ///
    /// Returns the latest chunk and optionally the Start chunk (if they differ).
    async fn fetch_initial_chunks(
        &mut self,
        volume: VolumeIndex,
    ) -> Result<(DownloadedChunk, Option<DownloadedChunk>)> {
        // Join the volume at its latest chunk
        let chunks = list_chunks_in_volume(self.config.site(), volume, usize::MAX).await?;
        self.requests_made += 1;

        let latest_id = chunks
            .last()
            .ok_or(Error::Aws(AwsError::ExpectedChunkNotFound))?;
        let latest_chunk = download_chunk(self.config.site(), latest_id).await?;
        self.requests_made += 1;
        self.bytes_downloaded += latest_chunk.chunk.data().len() as u64;

        let mut start_chunk = None;

        // If the latest chunk is a Start chunk, extract VCP from it
        if latest_chunk.identifier.chunk_type() == ChunkType::Start {
            self.refresh_vcp(&latest_chunk.chunk);
        } else {
            // Joined mid-volume: fetch the Start chunk for VCP
            let start_id = latest_chunk.identifier.with_sequence(1, ChunkType::Start);

            if let Ok(downloaded) = download_chunk(self.config.site(), &start_id).await {
                self.requests_made += 1;
                self.bytes_downloaded += downloaded.chunk.data().len() as u64;

                self.refresh_vcp(&downloaded.chunk);

                start_chunk = Some(downloaded);
            }
        }

        self.last_chunk_time = latest_chunk.identifier.upload_date_time();
        self.state = IteratorState::Ready(latest_chunk.identifier.clone());

        Ok((latest_chunk, start_chunk))
    }

    /// Returns the estimated time at which the next chunk becomes available for download from the
    /// real-time S3 bucket.
    ///
    /// The estimate is anchored to the current chunk's upload time, so it may already be in the
    /// past if the next chunk is due. Returns `None` if timing cannot be estimated (e.g., VCP not
    /// yet known). The caller can use this to schedule when to call [`try_next`](Self::try_next).
    pub fn next_expected_time(&self) -> Option<DateTime<Utc>> {
        let chunk_id = match &self.state {
            IteratorState::NeedVolumeStart(_) => return None,
            IteratorState::Ready(id) => id,
        };

        let vcp = self.vcp.as_ref()?;
        let mapper = self.elevation_mapper.as_ref()?;

        estimate_chunk_availability_time(chunk_id, vcp, mapper, Some(&self.timing_stats))
    }

    /// Returns the estimated duration from now until the next chunk becomes available for download
    /// (see [`next_expected_time`](Self::next_expected_time)).
    ///
    /// The result is a [`std::time::Duration`], the same type taken by the `sleep` function passed
    /// to [`chunk_stream`](super::chunk_stream) and by [`RetryPolicy`](super::RetryPolicy), so it
    /// can be handed directly to a sleep. Returns `None` if timing cannot be estimated or if the
    /// chunk should already be available.
    pub fn time_until_next(&self) -> Option<std::time::Duration> {
        let expected = self.next_expected_time()?;
        (expected - Utc::now()).to_std().ok()
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
            IteratorState::NeedVolumeStart(volume) => self.try_fetch_volume_start(*volume).await,
            IteratorState::Ready(current) => {
                let next = current
                    .next_chunk(
                        self.elevation_mapper
                            .as_ref()
                            .ok_or(AwsError::FailedToDetermineNextChunk)?,
                    )
                    .ok_or(AwsError::FailedToDetermineNextChunk)?;

                match next {
                    NextChunk::Sequence(next_id) => self.try_fetch_chunk(next_id).await,
                    NextChunk::Volume(next_volume) => {
                        self.try_fetch_volume_start(next_volume).await
                    }
                }
            }
        }
    }

    /// Attempts to fetch the Start chunk of a new volume.
    ///
    /// A new volume is always consumed from its Start chunk, even if later chunks were published
    /// while waiting, so that the volume header and VCP are yielded and the elevation mapper is
    /// refreshed for the new volume. Subsequent [`try_next`](Self::try_next) calls continue from
    /// sequence 2 in order.
    async fn try_fetch_volume_start(
        &mut self,
        volume: VolumeIndex,
    ) -> Result<Option<DownloadedChunk>> {
        let mut retry_state = RetryState::new(self.config.discovery_policy().clone());

        while retry_state.should_retry() {
            match self.fetch_volume_start(volume).await {
                Ok(Some(downloaded)) => {
                    // Update VCP and elevation mapper from the new volume's Start chunk
                    self.refresh_vcp(&downloaded.chunk);

                    self.record_arrival(&downloaded.identifier, downloaded.attempts);

                    return Ok(Some(downloaded));
                }
                Ok(None) => {
                    // The new volume or its Start chunk is not available yet, will retry
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
        Err(Error::Aws(AwsError::ExpectedChunkNotFound))
    }

    /// Lists a volume and downloads its Start chunk if the volume holds a scan newer than the
    /// current one.
    ///
    /// Returns `Ok(None)` if the volume has no newer scan yet or its Start chunk is not yet
    /// available.
    async fn fetch_volume_start(&mut self, volume: VolumeIndex) -> Result<Option<DownloadedChunk>> {
        let chunks = list_chunks_in_volume(self.config.site(), volume, usize::MAX).await?;
        self.requests_made += 1;

        let start_id = match next_volume_start(self.current(), &chunks) {
            Some(id) => id,
            None => return Ok(None),
        };

        self.requests_made += 1;
        match download_chunk(self.config.site(), &start_id).await {
            Ok(downloaded) => {
                self.bytes_downloaded += downloaded.chunk.data().len() as u64;
                Ok(Some(downloaded))
            }
            Err(Error::Aws(AwsError::S3ObjectNotFound)) => {
                debug!("Start chunk {} not yet available", start_id.name());
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Attempts to fetch a specific chunk.
    ///
    /// This is a single-attempt fetch for pull-based iteration. Returns:
    /// - `Ok(Some(chunk))` if successfully downloaded
    /// - `Ok(None)` if chunk is not yet available (caller should wait and retry)
    /// - `Err(...)` for unrecoverable errors
    async fn try_fetch_chunk(
        &mut self,
        chunk_id: ChunkIdentifier,
    ) -> Result<Option<DownloadedChunk>> {
        self.requests_made += 1;

        match download_chunk(self.config.site(), &chunk_id).await {
            Ok(downloaded) => {
                let identifier = &downloaded.identifier;
                self.bytes_downloaded += downloaded.chunk.data().len() as u64;

                // Update VCP if this is a start chunk
                if identifier.chunk_type() == ChunkType::Start {
                    self.refresh_vcp(&downloaded.chunk);
                }

                self.record_arrival(identifier, 1);

                Ok(Some(downloaded))
            }
            Err(Error::Aws(AwsError::S3ObjectNotFound)) => {
                // Chunk not yet available
                debug!("Chunk {} not yet available", chunk_id.name());
                Ok(None)
            }
            Err(e) => {
                debug!("Error downloading chunk: {:?}", e);
                Err(e)
            }
        }
    }

    /// Replaces the VCP and elevation mapper with those carried by a Start chunk. Leaves them
    /// unchanged if the chunk carries no decodable VCP.
    fn refresh_vcp(&mut self, chunk: &Chunk) {
        let Chunk::Start(file) = chunk else {
            return;
        };

        match file.coverage_pattern_message() {
            Ok(vcp) => {
                self.elevation_mapper = Some(ElevationChunkMapper::new(&vcp));
                self.vcp = Some(vcp);
            }
            Err(e) => debug!("Failed to decode VCP from start chunk: {:?}", e),
        }
    }

    /// Makes `identifier` the current chunk. The interval since the previous chunk's upload is
    /// added to the timing statistics under the arriving chunk's characteristics, the key under
    /// which [`next_expected_time`](Self::next_expected_time) looks it up to predict this chunk
    /// from its predecessor.
    fn record_arrival(&mut self, identifier: &ChunkIdentifier, attempts: usize) {
        // The hop from the Start chunk to sequence 2 is estimated with a fixed gap and never read
        // back from the statistics, so it is not recorded.
        let previous_is_start = self
            .current()
            .is_some_and(|current| current.chunk_type() == ChunkType::Start);
        if let (false, Some(upload_time), Some(prev_time), Some(vcp), Some(mapper)) = (
            previous_is_start,
            identifier.upload_date_time(),
            self.last_chunk_time,
            &self.vcp,
            &self.elevation_mapper,
        ) {
            if let Some(characteristics) = ChunkCharacteristics::for_chunk(
                identifier.sequence(),
                identifier.chunk_type(),
                vcp,
                mapper,
            ) {
                self.timing_stats
                    .add_timing(characteristics, upload_time - prev_time, attempts);
            }
        }

        self.last_chunk_time = identifier.upload_date_time();
        self.state = IteratorState::Ready(identifier.clone());
    }

    /// Returns the current chunk identifier, if available.
    pub fn current(&self) -> Option<&ChunkIdentifier> {
        match &self.state {
            IteratorState::Ready(id) => Some(id),
            IteratorState::NeedVolumeStart(_) => None,
        }
    }

    /// Returns the current VCP if available.
    pub fn vcp(&self) -> Option<&volume_coverage_pattern::Message<'static>> {
        self.vcp.as_ref()
    }

    /// Returns the site identifier.
    pub fn site(&self) -> &str {
        self.config.site()
    }

    /// Returns the configuration this iterator was created with.
    pub fn config(&self) -> &PollConfig {
        &self.config
    }

    /// Returns the total number of HTTP requests made by this iterator.
    ///
    /// This includes requests made during initialization and all subsequent
    /// `try_next()` calls. Useful for displaying network statistics in UIs.
    pub fn requests_made(&self) -> usize {
        self.requests_made
    }

    /// Returns the total bytes downloaded by this iterator.
    ///
    /// This includes chunk data downloaded during initialization and all
    /// subsequent `try_next()` calls.
    pub fn bytes_downloaded(&self) -> u64 {
        self.bytes_downloaded
    }

    /// Returns the projected time at which the current volume's final chunk becomes available for
    /// download from the real-time S3 bucket.
    ///
    /// Returns `None` if a projection cannot be made, for example before the VCP is known or when
    /// the current chunk is already the volume's final chunk.
    pub fn projected_volume_end_time(&self) -> Option<DateTime<Utc>> {
        let chunk_id = match &self.state {
            IteratorState::Ready(id) => id,
            IteratorState::NeedVolumeStart(_) => return None,
        };

        let vcp = self.vcp.as_ref()?;
        let mapper = self.elevation_mapper.as_ref()?;

        project_volume_end_time(chunk_id, vcp, mapper, Some(&self.timing_stats))
    }
}

fn advances_volume(current: Option<&ChunkIdentifier>, candidate: &ChunkIdentifier) -> bool {
    current.is_none_or(|current| candidate.date_time_prefix() > current.date_time_prefix())
}

/// Identifies the Start chunk to fetch when rolling over into a new volume.
///
/// `listed` is the next volume's listing from [`list_chunks_in_volume`], which holds only the
/// volume's newest scan in sequence order. Returns `None` while the listing is empty or holds a
/// scan no newer than `current`: volume numbers are reused after 999, so the next volume can still
/// contain its previous rotation. Otherwise returns the sequence 1 Start chunk of the listed scan,
/// regardless of how many later chunks are already listed, so the volume is consumed in order
/// from its header and VCP.
pub(crate) fn next_volume_start(
    current: Option<&ChunkIdentifier>,
    listed: &[ChunkIdentifier],
) -> Option<ChunkIdentifier> {
    let newest = listed.last()?;
    advances_volume(current, newest).then(|| newest.with_sequence(1, ChunkType::Start))
}

#[cfg(test)]
mod generation_regression_tests {
    use super::*;
    fn id(volume: usize, stamp: &str) -> ChunkIdentifier {
        ChunkIdentifier::from_name(
            "KBYX",
            VolumeIndex::new(volume),
            format!("{stamp}-001-S"),
            None,
        )
        .unwrap()
    }
    #[test]
    fn waits_for_new_generation_in_reused_next_volume() {
        let current = id(588, "20260912-140000");
        assert!(!advances_volume(
            Some(&current),
            &id(589, "20260909-013029")
        ));
        assert!(!advances_volume(
            Some(&current),
            &id(589, "20260912-140000")
        ));
        assert!(advances_volume(Some(&current), &id(589, "20260912-140452")));
    }
    #[test]
    fn volume_number_wrap_does_not_reverse_scan_time() {
        let current = id(999, "20260912-140000");
        assert!(advances_volume(Some(&current), &id(1, "20260912-140452")));
        assert!(advances_volume(None, &current));
    }
}

#[cfg(test)]
mod timing_tests {
    use super::*;
    use crate::aws::realtime::tests::get_test_vcp;
    use chrono::{Duration, NaiveDateTime, TimeZone};

    fn chunk_at(sequence: usize, upload: DateTime<Utc>) -> ChunkIdentifier {
        ChunkIdentifier::new(
            "KDMX",
            VolumeIndex::new(1),
            NaiveDateTime::parse_from_str("20220305_232324", "%Y%m%d_%H%M%S").unwrap(),
            sequence,
            ChunkType::Intermediate,
            Some(upload),
        )
    }

    #[test]
    fn interval_recorded_on_arrival_predicts_that_chunk() {
        let vcp = get_test_vcp();
        let mapper = ElevationChunkMapper::new(&vcp);

        // The first chunk of the second sweep arrives 7 seconds after the last chunk of the first
        let arriving_sequence = (3..=mapper.final_sequence())
            .find(|&sequence| {
                mapper
                    .get_chunk_metadata(sequence)
                    .unwrap()
                    .is_first_in_sweep()
            })
            .unwrap();
        let upload = Utc.with_ymd_and_hms(2022, 3, 5, 23, 30, 0).unwrap();
        let previous = chunk_at(arriving_sequence - 1, upload);
        let arriving = chunk_at(arriving_sequence, upload + Duration::seconds(7));

        let mut iterator = ChunkIterator::from_chunk(PollConfig::new("KDMX"), previous.clone());
        iterator.vcp = Some(vcp);
        iterator.elevation_mapper = Some(mapper);
        iterator.last_chunk_time = previous.upload_date_time();

        let physics_estimate = iterator.next_expected_time().unwrap();
        assert_ne!(physics_estimate, upload + Duration::seconds(7));

        iterator.record_arrival(&arriving, 1);
        assert_eq!(iterator.current(), Some(&arriving));

        // Predicting the arriving chunk from its predecessor reads the recorded interval
        iterator.state = IteratorState::Ready(previous);
        assert_eq!(
            iterator.next_expected_time(),
            Some(upload + Duration::seconds(7))
        );
    }

    #[test]
    fn start_to_first_intermediate_hop_is_not_recorded() {
        let vcp = get_test_vcp();
        let mapper = ElevationChunkMapper::new(&vcp);
        let upload = Utc.with_ymd_and_hms(2022, 3, 5, 23, 30, 0).unwrap();
        let start = ChunkIdentifier::new(
            "KDMX",
            VolumeIndex::new(1),
            NaiveDateTime::parse_from_str("20220305_232324", "%Y%m%d_%H%M%S").unwrap(),
            1,
            ChunkType::Start,
            Some(upload),
        );
        let second = chunk_at(2, upload + Duration::milliseconds(1500));

        let mut iterator = ChunkIterator::from_chunk(PollConfig::new("KDMX"), start.clone());
        iterator.vcp = Some(vcp);
        iterator.elevation_mapper = Some(mapper);
        iterator.last_chunk_time = start.upload_date_time();

        iterator.record_arrival(&second, 1);
        assert_eq!(iterator.current(), Some(&second));
        assert_eq!(iterator.last_chunk_time, second.upload_date_time());

        let characteristics = ChunkCharacteristics::for_chunk(
            2,
            ChunkType::Intermediate,
            iterator.vcp.as_ref().unwrap(),
            iterator.elevation_mapper.as_ref().unwrap(),
        )
        .unwrap();
        assert_eq!(
            iterator.timing_stats.get_average_timing(&characteristics),
            None
        );
    }
}
