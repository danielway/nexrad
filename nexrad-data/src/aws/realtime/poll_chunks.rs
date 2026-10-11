//! Real-time polling for NEXRAD radar chunks.
//!
//! This module provides a stream-based API (`chunk_stream`) for consuming
//! real-time NEXRAD data.

use crate::aws::realtime::{
    download_chunk, estimate_chunk_availability_time, get_latest_volume, list_chunks_in_volume,
    Chunk, ChunkCharacteristics, ChunkIdentifier, ChunkTimingStats, DownloadedChunk,
    ElevationChunkMapper, NextChunk, RetryPolicy, RetryState, VolumeIndex,
};
use crate::result::Error;
use crate::result::{aws::AWSError, Result};
use chrono::{Duration, Utc};
use futures_util::stream::Stream;
use log::debug;
use nexrad_decode::messages::volume_coverage_pattern;
use std::future::Future;

/// Configuration for polling real-time NEXRAD chunks.
#[derive(Debug, Clone)]
pub struct PollConfig {
    /// The radar site identifier (e.g., "KDMX").
    pub site: String,
    /// Retry policy for downloading individual chunks.
    pub download_policy: RetryPolicy,
    /// Retry policy for discovering new volumes.
    pub discovery_policy: RetryPolicy,
}

impl PollConfig {
    /// Creates a new poll configuration for the specified site with default policies.
    pub fn new(site: impl Into<String>) -> Self {
        Self {
            site: site.into(),
            download_policy: RetryPolicy::default_download(),
            discovery_policy: RetryPolicy::default_discovery(),
        }
    }

    /// Sets the retry policy for downloading individual chunks.
    pub fn with_download_policy(mut self, policy: RetryPolicy) -> Self {
        self.download_policy = policy;
        self
    }

    /// Sets the retry policy for discovering new volumes.
    pub fn with_discovery_policy(mut self, policy: RetryPolicy) -> Self {
        self.discovery_policy = policy;
        self
    }
}

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
}

/// Returns a stream of downloaded chunks for the specified configuration.
///
/// This is the preferred API for consuming real-time NEXRAD data. It returns
/// a [`Stream`] that yields chunks as they become available.
///
/// The stream is runtime-agnostic: it waits between polls by calling the
/// caller-supplied `sleep` function, which receives the duration to wait and
/// returns a future that completes once that duration has elapsed. With tokio,
/// pass `|d| tokio::time::sleep(d)`. On `wasm32` targets, a timer crate such as
/// `gloo-timers` provides an equivalent sleep future.
///
/// The returned stream is `Send` whenever `S` and `F` are `Send`.
///
/// # Example
///
/// ```ignore
/// use futures_util::StreamExt;
/// use nexrad_data::aws::realtime::{chunk_stream, PollConfig};
///
/// let config = PollConfig::new("KDMX");
/// let stream = chunk_stream(config, |d| tokio::time::sleep(d));
/// let mut stream = std::pin::pin!(stream);
///
/// while let Some(result) = stream.next().await {
///     match result {
///         Ok(chunk) => println!("Got chunk: {}", chunk.identifier.name()),
///         Err(e) => eprintln!("Error: {:?}", e),
///     }
/// }
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
            // Initialize state if needed
            let mut state = match state {
                Some(s) => s,
                None => match init_stream_state(&config).await {
                    Ok(s) => s,
                    Err(e) => return Some((Err(e), (config, sleep, None))),
                },
            };

            // Fetch the next chunk
            let result = fetch_next_chunk(&mut state, &mut sleep).await;
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
    use crate::aws::realtime::ChunkType;

    let latest_volume_result = get_latest_volume(&config.site).await?;
    let latest_volume = latest_volume_result
        .volume
        .ok_or(AWSError::LatestVolumeNotFound)?;

    let latest_chunk_id = get_latest_chunk(&config.site, latest_volume)
        .await?
        .ok_or(AWSError::ExpectedChunkNotFound)?;

    let (latest_chunk_id, _) = download_chunk(&config.site, &latest_chunk_id).await?;

    // Get VCP from start chunk
    let latest_metadata_id = ChunkIdentifier::new(
        config.site.clone(),
        latest_volume,
        *latest_chunk_id.date_time_prefix(),
        1,
        ChunkType::Start,
        None,
    );
    let (_, latest_metadata) = download_chunk(&config.site, &latest_metadata_id).await?;
    let vcp = get_latest_vcp(&latest_metadata)?;
    let elevation_mapper = ElevationChunkMapper::new(&vcp);

    debug!(
        "Stream initialized for {} at volume {} with VCP {}",
        config.site,
        latest_volume.as_number(),
        vcp.header().pattern_number()
    );

    Ok(ChunkStreamState {
        site: config.site.clone(),
        current_chunk: latest_chunk_id,
        elevation_mapper,
        vcp,
        timing_stats: ChunkTimingStats::new(),
        previous_chunk_time: None,
        download_policy: config.download_policy.clone(),
        discovery_policy: config.discovery_policy.clone(),
    })
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
    use crate::aws::realtime::ChunkType;

    // Estimate when the next chunk will be available
    let next_chunk_estimate = estimate_chunk_availability_time(
        &state.current_chunk,
        &state.vcp,
        &state.elevation_mapper,
        Some(&state.timing_stats),
    );

    if let Some(estimate) = next_chunk_estimate {
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
        .ok_or(AWSError::FailedToDetermineNextChunk)?
    {
        NextChunk::Sequence(next_id) => next_id,
        NextChunk::Volume(next_volume) => {
            let (_, chunk_id) = try_resiliently_with_policy(
                || get_latest_chunk_or_error(&state.site, next_volume),
                &state.discovery_policy,
                sleep,
            )
            .await;

            chunk_id.ok_or(AWSError::ExpectedChunkNotFound)?
        }
    };

    // Download the chunk with retries
    let (attempts, result) = try_resiliently_with_policy(
        || download_chunk(&state.site, &next_chunk_id),
        &state.download_policy,
        sleep,
    )
    .await;

    let (chunk_id, chunk) = result.ok_or(AWSError::ExpectedChunkNotFound)?;

    // Update timing statistics
    if let (Some(upload_time), Some(prev_time)) =
        (chunk_id.upload_date_time(), state.previous_chunk_time)
    {
        let duration = upload_time - prev_time;
        update_timing_stats(
            &mut state.timing_stats,
            &state.current_chunk,
            &state.vcp,
            &state.elevation_mapper,
            duration,
            attempts,
        );
    }

    // Update VCP if this is a start chunk
    if chunk_id.chunk_type() == ChunkType::Start {
        if let Ok(vcp) = get_latest_vcp(&chunk) {
            debug!(
                "Updated VCP to {} from start chunk",
                vcp.header().pattern_number()
            );
            state.elevation_mapper = ElevationChunkMapper::new(&vcp);
            state.vcp = vcp;
        }
    }

    // Update state for next iteration
    state.previous_chunk_time = chunk_id.upload_date_time();
    state.current_chunk = chunk_id.clone();

    Ok(DownloadedChunk {
        identifier: chunk_id,
        chunk,
        attempts,
    })
}

/// Helper function to update timing statistics for a downloaded chunk
fn update_timing_stats(
    timing_stats: &mut ChunkTimingStats,
    chunk_id: &ChunkIdentifier,
    vcp: &volume_coverage_pattern::Message,
    elevation_chunk_mapper: &ElevationChunkMapper,
    duration: Duration,
    attempts: usize,
) {
    if let Some(elevation) = elevation_chunk_mapper
        .get_sequence_elevation_number(chunk_id.sequence())
        .and_then(|elevation_number| vcp.elevations().get(elevation_number - 1))
    {
        let characteristics = ChunkCharacteristics {
            chunk_type: chunk_id.chunk_type(),
            waveform_type: elevation.waveform_type(),
            channel_configuration: elevation.channel_configuration(),
        };

        timing_stats.add_timing(characteristics, duration, attempts);
        debug!(
            "Updated timing statistics for {:?}: {}s",
            &characteristics as &dyn std::fmt::Debug,
            &(duration.num_milliseconds() as f64 / 1000.0) as &dyn std::fmt::Display,
        );
    }
}

/// Queries for the latest chunk in the specified volume. If no chunk is found, an error is returned.
async fn get_latest_chunk_or_error(site: &str, volume: VolumeIndex) -> Result<ChunkIdentifier> {
    let chunks = list_chunks_in_volume(site, volume, 100).await?;
    chunks
        .last()
        .cloned()
        .ok_or(Error::AWS(AWSError::ExpectedChunkNotFound))
}

/// Queries for the latest chunk in the specified volume.
async fn get_latest_chunk(site: &str, volume: VolumeIndex) -> Result<Option<ChunkIdentifier>> {
    let chunks = list_chunks_in_volume(site, volume, 100).await?;
    Ok(chunks.last().cloned())
}

/// Gets the volume coverage pattern from the latest metadata chunk.
fn get_latest_vcp(
    latest_metadata: &Chunk<'_>,
) -> Result<volume_coverage_pattern::Message<'static>> {
    if let Chunk::Start(file) = latest_metadata {
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

        sleep(to_std_duration(delay)).await;
    }

    (state.attempt(), None)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    fn assert_send<T: Send>(_: &T) {}

    #[test]
    fn chunk_stream_is_send_with_tokio_sleep() {
        let stream = chunk_stream(PollConfig::new("KDMX"), |d| tokio::time::sleep(d));
        assert_send(&stream);
    }
}
