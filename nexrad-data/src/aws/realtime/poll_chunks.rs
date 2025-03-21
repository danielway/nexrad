use crate::aws::realtime::poll_stats::PollStats;
use crate::aws::realtime::{
    download_chunk, estimate_next_chunk_time, get_latest_volume, list_chunks_in_volume, Chunk,
    ChunkCharacteristics, ChunkIdentifier, ChunkTimingStats, ChunkType, NewChunkStats, NextChunk,
    VolumeIndex,
};
use crate::result::Error;
use crate::result::{aws::AWSError, Result};
use chrono::{Duration, Utc};
use std::future::Future;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration as StdDuration;
use tokio::time::{sleep, sleep_until, Instant};

/// The number of chunks to wait before emitting timing statistics.
const CHUNKS_UNTIL_TIMING_STATS: usize = 10;

/// Polls for the latest real-time chunks from the AWS S3 bucket. When new chunks are identified,
/// they will be downloaded and sent to the provided `Sender`. If a statistics `Sender` is provided,
/// statistics from the polling process such as how many requests are being sent will be sent to it.
/// The polling process will stop when a message is received on the provided `Receiver`.
#[cfg(feature = "nexrad-decode")]
pub async fn poll_chunks(
    site: &str,
    tx: Sender<(ChunkIdentifier, Chunk<'_>)>,
    stats_tx: Option<Sender<PollStats>>,
    stop_rx: Receiver<bool>,
) -> Result<()> {
    use crate::aws::realtime::ChunkType;
    use log::debug;

    let latest_volume_result = get_latest_volume(site).await?;
    if let Some(stats_tx) = &stats_tx {
        stats_tx
            .send(PollStats::LatestVolumeCalls(latest_volume_result.calls))
            .map_err(|_| AWSError::PollingAsyncError)?;
    }

    let latest_volume = latest_volume_result
        .volume
        .ok_or(AWSError::LatestVolumeNotFound)?;

    let latest_chunk_id = get_latest_chunk(site, latest_volume)
        .await?
        .ok_or(AWSError::ExpectedChunkNotFound)?;

    let (latest_chunk_id, latest_chunk) = download_chunk(site, &latest_chunk_id).await?;
    tx.send((latest_chunk_id.clone(), latest_chunk))
        .map_err(|_| AWSError::PollingAsyncError)?;

    let (_, latest_metadata) = download_chunk(site, &latest_chunk_id.with_sequence(1)).await?;
    let mut vcp = get_volume_coverage_pattern(&latest_metadata)?;
    debug!("Polling volume with VCP: {}", vcp.header.pattern_number);

    // Create timing statistics for improved predictions
    let mut timing_stats = ChunkTimingStats::new();

    let mut previous_chunk_id = latest_chunk_id;
    let mut previous_chunk_time = None;

    let mut chunks_until_timing_stats = CHUNKS_UNTIL_TIMING_STATS;

    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }

        let next_chunk_estimate =
            estimate_next_chunk_time(&previous_chunk_id, &vcp, Some(&timing_stats));

        let next_chunk_time = if let Some(next_chunk_estimate) = next_chunk_estimate {
            debug!(
                "Estimated next chunk time: {} ({}s)",
                next_chunk_estimate,
                next_chunk_estimate
                    .signed_duration_since(Utc::now())
                    .num_milliseconds() as f64
                    / 1000.0
            );
            next_chunk_estimate
        } else {
            debug!("Unable to estimate next chunk time, trying immediately");
            Utc::now()
        };

        if next_chunk_time > Utc::now() {
            let time_until = next_chunk_time
                .signed_duration_since(Utc::now())
                .to_std()
                .ok();
            if let Some(time_until) = time_until {
                sleep_until(Instant::now() + time_until).await;
            }
        }

        let next_chunk_id = match previous_chunk_id
            .next_chunk()
            .ok_or(AWSError::FailedToDetermineNextChunk)?
        {
            NextChunk::Sequence(next_chunk_id) => next_chunk_id,
            NextChunk::Volume(next_volume) => {
                let (attempts, chunk_id) =
                    try_resiliently(|| get_latest_chunk_or_error(site, next_volume), 500, 10).await;

                if let Some(stats_tx) = &stats_tx {
                    stats_tx
                        .send(PollStats::NewVolumeCalls(attempts))
                        .map_err(|_| AWSError::PollingAsyncError)?;
                }

                chunk_id.ok_or(AWSError::ExpectedChunkNotFound)?
            }
        };

        let (attempts, next_chunk) =
            try_resiliently(|| download_chunk(site, &next_chunk_id), 500, 5).await;

        let (next_chunk_id, next_chunk) = next_chunk.ok_or(AWSError::ExpectedChunkNotFound)?;

        if let (Some(chunk_time), Some(previous_chunk_time)) =
            (next_chunk_id.date_time(), previous_chunk_time)
        {
            let chunk_duration = chunk_time.signed_duration_since(previous_chunk_time);
            update_timing_stats(
                &mut timing_stats,
                &previous_chunk_id,
                &vcp,
                chunk_duration,
                attempts,
            );
        }

        if next_chunk_id.chunk_type() == Some(ChunkType::Start) {
            vcp = get_volume_coverage_pattern(&next_chunk)?;
            debug!(
                "Updated polling volume's VCP to: {}",
                vcp.header.pattern_number
            );
        }

        if let Some(stats_tx) = &stats_tx {
            stats_tx
                .send(PollStats::NewChunk(NewChunkStats {
                    calls: attempts,
                    download_time: Some(Utc::now()),
                    upload_time: next_chunk_id.date_time(),
                }))
                .map_err(|_| AWSError::PollingAsyncError)?;

            if chunks_until_timing_stats == 0 {
                stats_tx
                    .send(PollStats::ChunkTimings(timing_stats))
                    .map_err(|_| AWSError::PollingAsyncError)?;
                timing_stats = ChunkTimingStats::new();
                chunks_until_timing_stats = CHUNKS_UNTIL_TIMING_STATS;
            } else {
                chunks_until_timing_stats -= 1;
            }
        }

        tx.send((next_chunk_id.clone(), next_chunk))
            .map_err(|_| AWSError::PollingAsyncError)?;

        previous_chunk_time = next_chunk_id.date_time();
        previous_chunk_id = next_chunk_id;
    }

    Ok(())
}

/// Helper function to update timing statistics for a downloaded chunk
#[cfg(feature = "nexrad-decode")]
fn update_timing_stats(
    timing_stats: &mut ChunkTimingStats,
    chunk_id: &ChunkIdentifier,
    vcp: &nexrad_decode::messages::volume_coverage_pattern::Message,
    duration: Duration,
    attempts: usize,
) {
    use log::debug;

    if let Some(sequence) = chunk_id.sequence() {
        if let Some(elevation) = super::get_elevation_from_chunk(sequence, &vcp.elevations) {
            let chunk_type = if sequence == 1 {
                ChunkType::Start
            } else if sequence == 55 {
                ChunkType::End
            } else {
                ChunkType::Intermediate
            };

            let characteristics = ChunkCharacteristics {
                chunk_type,
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
#[cfg(feature = "nexrad-decode")]
fn get_volume_coverage_pattern(
    latest_metadata: &Chunk<'_>,
) -> Result<nexrad_decode::messages::volume_coverage_pattern::Message> {
    if let Chunk::Start(file) = latest_metadata {
        for mut record in file.records() {
            if record.compressed() {
                record = record.decompress()?;
            }

            for message in record.messages()? {
                if let nexrad_decode::messages::MessageContents::VolumeCoveragePattern(vcp) =
                    message.contents()
                {
                    return Ok(*vcp.clone());
                }
            }
        }
    }

    Err(Error::MissingCoveragePattern)
}

/// Attempts an action with retries on an exponential backoff.
async fn try_resiliently<F, R>(
    action: impl Fn() -> F,
    wait_millis: u64,
    attempts: usize,
) -> (usize, Option<R>)
where
    F: Future<Output = Result<R>>,
{
    for attempt in 0..attempts {
        if let Ok(result) = action().await {
            return (attempt + 1, Some(result));
        }

        let wait = wait_millis * 2u64.pow(attempt as u32);
        sleep(StdDuration::from_millis(wait)).await;
    }

    (attempts, None)
}
