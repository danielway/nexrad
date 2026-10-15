#![cfg(feature = "aws")]

use chrono::Utc;
use clap::Parser;
use env_logger::{Builder, Env};
use log::{info, LevelFilter};
use nexrad_data::aws::realtime::{Chunk, ChunkIterator, DownloadedChunk};
use nexrad_data::{Error, Result};
use std::time::Duration;

/// Example demonstrating the pull-based ChunkIterator for real-time NEXRAD data.
///
/// The ChunkIterator provides manual control over timing, making it suitable for
/// environments without tokio or where caller-controlled scheduling is preferred.
/// Unlike the streaming API, callers decide when to poll for the next chunk.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Site identifier (e.g., KDMX)
    #[arg(default_value = "KDMX")]
    site: String,

    /// Number of chunks to download
    #[arg(default_value = "5")]
    chunk_count: usize,

    /// Maximum wait time between chunks in seconds
    #[arg(long, default_value = "30")]
    max_wait: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info"))
        .filter_module("reqwest::connect", LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    info!("Creating ChunkIterator for site {}", cli.site);

    // Create the iterator - this discovers the latest volume and fetches initial chunks
    let init = ChunkIterator::start(&cli.site).await?;

    let mut iterator = init.iterator;

    info!("Iterator initialized, processing initial chunks...");

    // Process the Start chunk if we joined mid-volume
    // This contains the VCP (Volume Coverage Pattern) metadata
    if let Some(start_chunk) = init.start_chunk {
        info!("Received Start chunk (fetched separately for VCP metadata):");
        log_chunk(&start_chunk, 0, 0);
    }

    // Process the latest chunk (the chunk we joined at)
    info!("Received latest chunk (joined at):");
    log_chunk(&init.latest_chunk, 0, 0);

    // Display VCP information from the initial chunks
    if let Some(vcp) = iterator.vcp() {
        info!(
            "Volume Coverage Pattern: VCP{} ({} elevations)",
            vcp.header().vcp_number_raw(),
            vcp.elevations().len()
        );
    }

    let mut downloaded = 1; // Count the initial latest_chunk

    // Fetch additional chunks
    while downloaded < cli.chunk_count {
        // Check if we have timing information for the next chunk
        if let Some(expected_time) = iterator.next_expected_time() {
            info!("Next chunk expected at: {}", expected_time);

            if let Some(wait_duration) = iterator.time_until_next() {
                let actual_wait = wait_duration.min(Duration::from_secs(cli.max_wait));
                info!(
                    "Waiting {:.2}s for next chunk...",
                    actual_wait.as_secs_f64()
                );
                tokio::time::sleep(actual_wait).await;
            }
        }

        // Make a single attempt; the iterator never sleeps or retries, so the cadence is ours
        match iterator.try_next().await {
            Ok(Some(chunk)) => {
                downloaded += 1;
                log_chunk(&chunk, downloaded, cli.chunk_count);
            }
            Ok(None) => {
                info!("Chunk not yet available, waiting...");
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(Error::Aws(error)) if error.is_transient() => {
                info!("Transient error ({error}), waiting...");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(error) => return Err(error),
        }
    }

    // Display a session summary
    info!(
        "Network statistics: {} requests, {} bytes downloaded",
        iterator.requests_made(),
        iterator.bytes_downloaded()
    );
    match iterator.projected_volume_end_time() {
        Some(end_time) => info!("Final chunk of this volume expected at: {}", end_time),
        None => info!("Current volume completion time cannot be projected yet"),
    }

    info!("Done! Downloaded {} chunks", downloaded);

    Ok(())
}

fn log_chunk(chunk: &DownloadedChunk, current: usize, total: usize) {
    let download_time = Utc::now();

    let latency = chunk
        .identifier
        .upload_date_time()
        .map(|t| (download_time - t).num_milliseconds() as f64 / 1000.0);

    if current > 0 && total > 0 {
        info!(
            "[{}/{}] Downloaded chunk {} (type: {:?}, seq: {}, attempts: {}, latency: {:.2?}s)",
            current,
            total,
            chunk.identifier.name(),
            chunk.identifier.chunk_type(),
            chunk.identifier.sequence(),
            chunk.attempts,
            latency
        );
    } else {
        info!(
            "  Chunk {} (type: {:?}, seq: {}, latency: {:.2?}s)",
            chunk.identifier.name(),
            chunk.identifier.chunk_type(),
            chunk.identifier.sequence(),
            latency
        );
    }

    // Show chunk size
    let size = chunk.chunk.data().len();
    info!("  Size: {} bytes", size);

    // For start chunks, show additional information
    if let Chunk::Start(file) = &chunk.chunk {
        if let Ok(records) = file.records() {
            info!("  Records: {}", records.len());
        }
    }
}
