use crate::aws::realtime::poll_stats::PollStats;
use crate::aws::realtime::{
    download_chunk, estimate_next_chunk_time, get_latest_volume, list_chunks_in_volume, Chunk,
    ChunkIdentifier, NewChunkStats, NextChunk, VolumeIndex,
};
use crate::result::{aws::AWSError, Result};
use chrono::Utc;
use std::future::Future;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use tokio::time::{sleep, sleep_until, Instant};

/// Polls for the latest real-time chunks from the AWS S3 bucket. When new chunks are identified,
/// they will be downloaded and sent to the provided `Sender`. If a statistics `Sender` is provided,
/// statistics from the polling process such as how many requests are being sent will be sent to it.
/// The polling process will stop when a message is received on the provided `Receiver`.
pub async fn poll_chunks(
    site: &str,
    tx: Sender<(ChunkIdentifier, Chunk<'_>)>,
    stats_tx: Option<Sender<PollStats>>,
    stop_rx: Receiver<bool>,
) -> Result<()> {
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

    let mut previous_chunk_id = latest_chunk_id;
    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }

        let next_chunk_time = estimate_next_chunk_time(&previous_chunk_id);
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
                    try_resiliently(|| get_latest_chunk(site, next_volume), 500, 5).await;

                if let Some(stats_tx) = &stats_tx {
                    stats_tx
                        .send(PollStats::NewVolumeCalls(attempts))
                        .map_err(|_| AWSError::PollingAsyncError)?;
                }

                chunk_id.flatten().ok_or(AWSError::ExpectedChunkNotFound)?
            }
        };

        let (attempts, next_chunk) =
            try_resiliently(|| download_chunk(site, &next_chunk_id), 500, 5).await;

        let (next_chunk_id, next_chunk) = next_chunk.ok_or(AWSError::ExpectedChunkNotFound)?;

        if let Some(stats_tx) = &stats_tx {
            let latency = next_chunk_id
                .date_time()
                .and_then(|date_time| Utc::now().signed_duration_since(date_time).to_std().ok());

            stats_tx
                .send(PollStats::NewChunk(NewChunkStats {
                    calls: attempts,
                    latency,
                }))
                .map_err(|_| AWSError::PollingAsyncError)?;
        }

        tx.send((next_chunk_id.clone(), next_chunk))
            .map_err(|_| AWSError::PollingAsyncError)?;

        previous_chunk_id = next_chunk_id;
    }

    Ok(())
}

/// Queries for the latest chunk in the specified volume.
async fn get_latest_chunk(site: &str, volume: VolumeIndex) -> Result<Option<ChunkIdentifier>> {
    let chunks = list_chunks_in_volume(site, volume, 100).await?;
    Ok(chunks.last().cloned())
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
        sleep(Duration::from_millis(wait)).await;
    }

    (attempts, None)
}
