//! Real-time polling for NEXRAD radar chunks.
//!
//! This module provides a stream-based API (`chunk_stream`) for consuming
//! real-time NEXRAD data.

use crate::aws::realtime::{
    download_chunk, estimate_chunk_availability_time, get_latest_volume, list_chunks_in_volume,
    next_volume_start, Chunk, ChunkCharacteristics, ChunkIdentifier, ChunkTimingStats, ChunkType,
    DownloadedChunk, ElevationChunkMapper, NextChunk, PollConfig, RetryPolicy, RetryState,
    VolumeIndex,
};
use crate::result::Error;
use crate::result::{AwsError, Result};
use chrono::{Duration, Utc};
use futures_util::stream::Stream;
use log::debug;
use nexrad_decode::messages::volume_coverage_pattern;
use std::collections::VecDeque;
use std::future::Future;

/// Internal state for the chunk stream.
struct ChunkStreamState {
    site: String,
    current_chunk: ChunkIdentifier,
    elevation_mapper: ElevationChunkMapper,
    vcp: volume_coverage_pattern::Message<'static>,
    timing_stats: ChunkTimingStats,
    previous_chunk_time: Option<chrono::DateTime<Utc>>,
    download_policy: RetryPolicy,
    discovery_policy: RetryPolicy,
    /// Chunks downloaded during startup that have not been yielded yet.
    pending: VecDeque<DownloadedChunk>,
}

impl ChunkStreamState {
    /// Builds the stream state from the chunks downloaded at startup: the volume's latest chunk
    /// and, when that is not the Start chunk, the Start chunk fetched separately. The VCP comes
    /// from whichever of them is the Start chunk. Both are queued to be yielded, the Start chunk
    /// first, and the latest chunk becomes the current chunk with its upload time as the anchor
    /// for the first recorded interval.
    fn from_startup(
        config: &PollConfig,
        latest: DownloadedChunk,
        start: Option<DownloadedChunk>,
    ) -> Result<Self> {
        let vcp = match &start.as_ref().unwrap_or(&latest).chunk {
            Chunk::Start(file) => file.coverage_pattern_message()?,
            Chunk::IntermediateOrEnd(_) => return Err(Error::MissingCoveragePattern),
        };
        let elevation_mapper = ElevationChunkMapper::new(&vcp);

        Ok(Self {
            site: config.site().to_string(),
            current_chunk: latest.identifier.clone(),
            elevation_mapper,
            vcp,
            timing_stats: ChunkTimingStats::new(),
            previous_chunk_time: latest.identifier.upload_date_time(),
            download_policy: config.download_policy().clone(),
            discovery_policy: config.discovery_policy().clone(),
            pending: start.into_iter().chain([latest]).collect(),
        })
    }

    /// Returns the estimated time at which the chunk following the current chunk becomes
    /// available for download.
    fn next_chunk_estimate(&self) -> Option<chrono::DateTime<Utc>> {
        estimate_chunk_availability_time(
            &self.current_chunk,
            &self.vcp,
            &self.elevation_mapper,
            Some(&self.timing_stats),
        )
    }
}

/// Returns a stream of downloaded chunks for the specified configuration.
///
/// This is the preferred API for consuming real-time NEXRAD data. It returns
/// a [`Stream`] that yields chunks as they become available.
///
/// At startup the stream finds the latest volume and downloads its latest chunk, plus the volume's
/// Start chunk when the latest chunk is not the Start chunk. It yields the Start chunk first (when
/// it was fetched separately), then the latest chunk, then continues from the chunk after the
/// latest. When joining mid-volume, the first volume is therefore partial: the chunks between the
/// Start chunk and the latest chunk are not yielded.
///
/// The [`PollConfig`]'s download policy governs retries of each chunk download and its discovery
/// policy governs retries while waiting for the next volume.
///
/// The stream is runtime-agnostic: it waits between polls by calling the
/// caller-supplied `sleep` function, which receives the duration to wait and
/// returns a future that completes once that duration has elapsed. With tokio,
/// pass `|d| tokio::time::sleep(d)`. On `wasm32` targets, a timer crate such as
/// `gloo-timers` provides an equivalent sleep future.
///
/// The returned stream is `Send` whenever `S` and `F` are `Send`.
///
/// Within a volume, chunks are yielded in sequence order. When a volume ends, the stream waits for
/// the next volume and always yields its Start chunk (sequence 1, carrying the volume header and
/// VCP) first, then continues from sequence 2, even if later chunks were already published while
/// it waited. The stream's VCP and elevation mapping are refreshed from each Start chunk, so every
/// volume can be passed to `assemble_volume` and a VCP change at rollover is picked up.
///
/// # Example
///
/// ```no_run
/// use futures_util::StreamExt;
/// use nexrad_data::aws::realtime::{chunk_stream, PollConfig};
///
/// # async fn run() {
/// let config = PollConfig::new("KDMX");
/// let stream = chunk_stream(config, |d| tokio::time::sleep(d));
/// let mut stream = std::pin::pin!(stream);
///
/// while let Some(result) = stream.next().await {
///     match result {
///         Ok(chunk) => println!("Got chunk: {}", chunk.identifier.name()),
///         Err(e) => eprintln!("Error: {e}"),
///     }
/// }
/// # }
/// ```
pub fn chunk_stream<S, F>(
    config: PollConfig,
    sleep: S,
) -> impl Stream<Item = Result<DownloadedChunk>>
where
    S: Fn(std::time::Duration) -> F,
    F: Future<Output = ()>,
{
    futures_util::stream::unfold(
        (config, sleep, None),
        |(config, mut sleep, state): (PollConfig, S, Option<ChunkStreamState>)| async move {
            let mut state = match state {
                Some(s) => s,
                None => match init_stream_state(&config).await {
                    Ok(s) => s,
                    Err(e) => return Some((Err(e), (config, sleep, None))),
                },
            };

            // Yield the startup chunks, then fetch the next chunk
            let result = match state.pending.pop_front() {
                Some(chunk) => Ok(chunk),
                None => fetch_next_chunk(&mut state, &mut sleep).await,
            };
            Some((result, (config, sleep, Some(state))))
        },
    )
}

/// Converts a `chrono::Duration` into a `std::time::Duration`, clamping negative
/// durations to zero.
fn to_std_duration(duration: Duration) -> std::time::Duration {
    duration.to_std().unwrap_or(std::time::Duration::ZERO)
}

async fn init_stream_state(config: &PollConfig) -> Result<ChunkStreamState> {
    let latest_volume = get_latest_volume(config.site())
        .await?
        .volume
        .ok_or(AwsError::LatestVolumeNotFound)?;

    let latest_chunk_id = get_latest_chunk(config.site(), latest_volume)
        .await?
        .ok_or(AwsError::LatestVolumeNotFound)?;
    let latest = download_chunk(&latest_chunk_id).await?;

    // Joined mid-volume: the Start chunk carries the VCP
    let start = if latest.identifier.chunk_type() == ChunkType::Start {
        None
    } else {
        let start_id = latest.identifier.with_sequence(1, ChunkType::Start);
        Some(download_chunk(&start_id).await?)
    };

    let state = ChunkStreamState::from_startup(config, latest, start)?;
    debug!(
        "Stream initialized for {} at volume {} with VCP {}",
        config.site(),
        latest_volume.number(),
        state.vcp.header().vcp_number_raw()
    );

    Ok(state)
}

// The sleep function is passed as `&mut S` rather than `&S` so that the
// resulting future is `Send` whenever `S: Send`, without also requiring `S: Sync`.
async fn fetch_next_chunk<S, F>(
    state: &mut ChunkStreamState,
    sleep: &mut S,
) -> Result<DownloadedChunk>
where
    S: Fn(std::time::Duration) -> F,
    F: Future<Output = ()>,
{
    // Estimate when the next chunk will be available
    if let Some(estimate) = state.next_chunk_estimate() {
        let now = Utc::now();
        if estimate > now {
            let wait_duration = to_std_duration(estimate - now);
            debug!("Waiting {}ms for next chunk", wait_duration.as_millis());
            sleep(wait_duration).await;
        }
    }

    // Determine what to fetch next
    let next_chunk_id = match state
        .current_chunk
        .next_chunk(&state.elevation_mapper)
        .ok_or(AwsError::FailedToDetermineNextChunk)?
    {
        NextChunk::Sequence(next_id) => next_id,
        NextChunk::Volume(next_volume) => {
            let current_chunk = &state.current_chunk;
            let (_, chunk_id) = try_resiliently_with_policy(
                || get_volume_start_or_error(&state.site, next_volume, current_chunk),
                &state.discovery_policy,
                sleep,
            )
            .await;

            chunk_id.ok_or(AwsError::ExpectedChunkNotFound)?
        }
    };

    // Download the chunk with retries
    let (attempts, result) = try_resiliently_with_policy(
        || download_chunk(&next_chunk_id),
        &state.download_policy,
        sleep,
    )
    .await;

    let DownloadedChunk {
        identifier: chunk_id,
        chunk,
        ..
    } = result.ok_or(AwsError::ExpectedChunkNotFound)?;

    record_arrival(state, &chunk_id, &chunk, attempts);

    Ok(DownloadedChunk::new(chunk_id, chunk, attempts))
}

/// Makes `chunk_id` the stream's current chunk. The interval since the previous chunk's upload is
/// added to the timing statistics under the arriving chunk's characteristics, the key under which
/// the next-chunk estimate looks it up to predict this chunk from its predecessor. A Start chunk
/// also refreshes the VCP and elevation mapping.
fn record_arrival(
    state: &mut ChunkStreamState,
    chunk_id: &ChunkIdentifier,
    chunk: &Chunk,
    attempts: usize,
) {
    // The hop from the Start chunk to sequence 2 is estimated with a fixed gap and never read back
    // from the statistics, so it is not recorded.
    let previous_is_start = state.current_chunk.chunk_type() == ChunkType::Start;
    if let (false, Some(upload_time), Some(prev_time)) = (
        previous_is_start,
        chunk_id.upload_date_time(),
        state.previous_chunk_time,
    ) {
        if let Some(characteristics) = ChunkCharacteristics::for_chunk(
            chunk_id.sequence(),
            chunk_id.chunk_type(),
            &state.vcp,
            &state.elevation_mapper,
        ) {
            let duration = upload_time - prev_time;
            state
                .timing_stats
                .add_timing(characteristics, duration, attempts);
            debug!(
                "Updated timing statistics for {:?}: {}s",
                characteristics,
                duration.num_milliseconds() as f64 / 1000.0,
            );
        }
    }

    if chunk_id.chunk_type() == ChunkType::Start {
        if let Chunk::Start(file) = chunk {
            match file.coverage_pattern_message() {
                Ok(vcp) => {
                    debug!(
                        "Updated VCP to {} from start chunk",
                        vcp.header().vcp_number_raw()
                    );
                    state.elevation_mapper = ElevationChunkMapper::new(&vcp);
                    state.vcp = vcp;
                }
                Err(e) => debug!("Failed to decode VCP from start chunk: {:?}", e),
            }
        }
    }

    state.previous_chunk_time = chunk_id.upload_date_time();
    state.current_chunk = chunk_id.clone();
}

/// Identifies the Start chunk of the specified volume once it holds a scan newer than `current`.
/// If the volume has no newer scan yet, an error is returned so the caller retries.
async fn get_volume_start_or_error(
    site: &str,
    volume: VolumeIndex,
    current: &ChunkIdentifier,
) -> Result<ChunkIdentifier> {
    let chunks = list_chunks_in_volume(site, volume).await?;
    next_volume_start(Some(current), &chunks).ok_or(Error::Aws(AwsError::ExpectedChunkNotFound))
}

/// Queries for the latest chunk in the specified volume.
async fn get_latest_chunk(site: &str, volume: VolumeIndex) -> Result<Option<ChunkIdentifier>> {
    let chunks = list_chunks_in_volume(site, volume).await?;
    Ok(chunks.last().cloned())
}

/// Attempts an action with retries using the provided retry policy.
async fn try_resiliently_with_policy<A, R, S, F>(
    action: impl Fn() -> A,
    policy: &RetryPolicy,
    sleep: &mut S,
) -> (usize, Option<R>)
where
    A: Future<Output = Result<R>>,
    S: Fn(std::time::Duration) -> F,
    F: Future<Output = ()>,
{
    let mut state = RetryState::new(policy.clone());

    while let Some(delay) = state.next_delay() {
        if let Ok(result) = action().await {
            return (state.attempt(), Some(result));
        }

        sleep(delay).await;
    }

    (state.attempt(), None)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    fn assert_send<T: Send>(_: &T) {}

    #[test]
    fn interval_recorded_on_arrival_predicts_that_chunk() {
        use crate::aws::realtime::tests::get_test_vcp;
        use chrono::{NaiveDateTime, TimeZone};

        let vcp = get_test_vcp();
        let elevation_mapper = ElevationChunkMapper::new(&vcp);

        // The first chunk of the second sweep arrives 7 seconds after the last chunk of the first
        let arriving_sequence = (3..=elevation_mapper.final_sequence())
            .find(|&sequence| {
                elevation_mapper
                    .get_chunk_metadata(sequence)
                    .unwrap()
                    .is_first_in_sweep()
            })
            .unwrap();
        let upload = Utc.with_ymd_and_hms(2022, 3, 5, 23, 30, 0).unwrap();
        let chunk_at = |sequence, upload| {
            ChunkIdentifier::new(
                "KDMX",
                VolumeIndex::new(1).unwrap(),
                NaiveDateTime::parse_from_str("20220305_232324", "%Y%m%d_%H%M%S")
                    .unwrap()
                    .and_utc(),
                sequence,
                ChunkType::Intermediate,
                Some(upload),
            )
        };
        let previous = chunk_at(arriving_sequence - 1, upload);
        let arriving = chunk_at(arriving_sequence, upload + Duration::seconds(7));

        let mut state = ChunkStreamState {
            site: "KDMX".to_string(),
            current_chunk: previous.clone(),
            elevation_mapper,
            vcp,
            timing_stats: ChunkTimingStats::new(),
            previous_chunk_time: previous.upload_date_time(),
            download_policy: RetryPolicy::default_download(),
            discovery_policy: RetryPolicy::default_discovery(),
            pending: VecDeque::new(),
        };
        let physics_estimate = state.next_chunk_estimate().unwrap();
        assert_ne!(physics_estimate, upload + Duration::seconds(7));

        let chunk = Chunk::IntermediateOrEnd(crate::volume::Record::new(vec![0; 8]));
        record_arrival(&mut state, &arriving, &chunk, 1);
        assert_eq!(state.current_chunk, arriving);
        assert_eq!(state.previous_chunk_time, arriving.upload_date_time());

        // Predicting the arriving chunk from its predecessor reads the recorded interval
        state.current_chunk = previous;
        assert_eq!(
            state.next_chunk_estimate(),
            Some(upload + Duration::seconds(7))
        );
    }

    #[test]
    fn start_to_first_intermediate_hop_is_not_recorded() {
        use crate::aws::realtime::tests::get_test_vcp;
        use chrono::{NaiveDateTime, TimeZone};

        let vcp = get_test_vcp();
        let elevation_mapper = ElevationChunkMapper::new(&vcp);
        let upload = Utc.with_ymd_and_hms(2022, 3, 5, 23, 30, 0).unwrap();
        let prefix = NaiveDateTime::parse_from_str("20220305_232324", "%Y%m%d_%H%M%S")
            .unwrap()
            .and_utc();
        let start = ChunkIdentifier::new(
            "KDMX",
            VolumeIndex::new(1).unwrap(),
            prefix,
            1,
            ChunkType::Start,
            Some(upload),
        );
        let second = ChunkIdentifier::new(
            "KDMX",
            VolumeIndex::new(1).unwrap(),
            prefix,
            2,
            ChunkType::Intermediate,
            Some(upload + Duration::milliseconds(1500)),
        );

        let mut state = ChunkStreamState {
            site: "KDMX".to_string(),
            current_chunk: start.clone(),
            elevation_mapper,
            vcp,
            timing_stats: ChunkTimingStats::new(),
            previous_chunk_time: start.upload_date_time(),
            download_policy: RetryPolicy::default_download(),
            discovery_policy: RetryPolicy::default_discovery(),
            pending: VecDeque::new(),
        };

        let chunk = Chunk::IntermediateOrEnd(crate::volume::Record::new(vec![0; 8]));
        record_arrival(&mut state, &second, &chunk, 1);
        assert_eq!(state.current_chunk, second);
        assert_eq!(state.previous_chunk_time, second.upload_date_time());

        let characteristics = ChunkCharacteristics::for_chunk(
            2,
            ChunkType::Intermediate,
            &state.vcp,
            &state.elevation_mapper,
        )
        .unwrap();
        assert_eq!(
            state.timing_stats.get_average_timing(&characteristics),
            None
        );
    }

    fn downloaded(identifier: ChunkIdentifier, chunk: Chunk) -> DownloadedChunk {
        DownloadedChunk::new(identifier, chunk, 1)
    }

    fn startup_chunk(sequence: usize, chunk_type: ChunkType) -> ChunkIdentifier {
        use chrono::{NaiveDateTime, TimeZone};

        ChunkIdentifier::new(
            "KDMX",
            VolumeIndex::new(1).unwrap(),
            NaiveDateTime::parse_from_str("20220305_232324", "%Y%m%d_%H%M%S")
                .unwrap()
                .and_utc(),
            sequence,
            chunk_type,
            Some(
                Utc.with_ymd_and_hms(2022, 3, 5, 23, 25, sequence as u32)
                    .unwrap(),
            ),
        )
    }

    #[test]
    fn startup_yields_start_then_latest_and_seeds_previous_time() {
        use crate::aws::realtime::tests::{get_test_vcp, start_chunk_from_test_volume};

        let start = startup_chunk(1, ChunkType::Start);
        let latest = startup_chunk(12, ChunkType::Intermediate);
        let mut state = ChunkStreamState::from_startup(
            &PollConfig::new("KDMX"),
            downloaded(
                latest.clone(),
                Chunk::IntermediateOrEnd(crate::volume::Record::new(vec![0; 8])),
            ),
            Some(downloaded(start.clone(), start_chunk_from_test_volume(0))),
        )
        .unwrap();

        assert_eq!(state.vcp, get_test_vcp());
        assert_eq!(state.current_chunk, latest);
        assert_eq!(state.previous_chunk_time, latest.upload_date_time());
        assert_eq!(
            state.pending.pop_front().map(|chunk| chunk.identifier),
            Some(start)
        );
        assert_eq!(
            state.pending.pop_front().map(|chunk| chunk.identifier),
            Some(latest)
        );
        assert!(state.pending.is_empty());
    }

    #[test]
    fn startup_at_start_chunk_yields_it_once() {
        let start = startup_chunk(1, ChunkType::Start);
        let mut state = ChunkStreamState::from_startup(
            &PollConfig::new("KDMX"),
            downloaded(
                start.clone(),
                crate::aws::realtime::tests::start_chunk_from_test_volume(0),
            ),
            None,
        )
        .unwrap();

        assert_eq!(state.previous_chunk_time, start.upload_date_time());
        assert_eq!(
            state.pending.pop_front().map(|chunk| chunk.identifier),
            Some(start)
        );
        assert!(state.pending.is_empty());
    }

    #[test]
    fn startup_without_coverage_pattern_fails() {
        let latest = startup_chunk(12, ChunkType::Intermediate);
        let result = ChunkStreamState::from_startup(
            &PollConfig::new("KDMX"),
            downloaded(
                latest,
                Chunk::IntermediateOrEnd(crate::volume::Record::new(vec![0; 8])),
            ),
            None,
        );
        assert!(matches!(result, Err(Error::MissingCoveragePattern)));
    }

    #[test]
    fn chunk_stream_is_send_with_tokio_sleep() {
        let stream = chunk_stream(PollConfig::new("KDMX"), |d| tokio::time::sleep(d));
        assert_send(&stream);
    }
}
