use clap::Parser;
use log::{debug, info, warn, LevelFilter};
use std::collections::HashMap;
use std::sync::Mutex;

// Example designed to provide concise latency analysis for NEXRAD data chunks
// Output format (single line per chunk):
// Chunk: <name> | Downloaded: <time> | AWS Latency: <value>s | First Radial Latency: <value>s | Last Radial Latency: <value>s | Attempts: <count>

#[cfg(not(all(feature = "aws", feature = "decode")))]
fn main() {
    println!("This example requires the \"aws\" and \"decode\" features to be enabled.");
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Site identifier (e.g., KDMX)
    #[arg(default_value = "KDMX")]
    site: String,

    /// The number of chunks to download
    #[arg(default_value = "10")]
    chunk_count: usize,
}

/// Track a chunk's download attempt information
struct ChunkDownloadInfo {
    attempts: usize,
    processed: bool,
}

#[cfg(all(feature = "aws", feature = "decode"))]
#[tokio::main]
async fn main() -> nexrad_data::result::Result<()> {
    use chrono::Utc;
    use nexrad_data::aws::realtime::Chunk;
    use nexrad_data::aws::realtime::{poll_chunks, ChunkIdentifier, PollStats};
    use std::sync::mpsc;
    use std::time::Duration;
    use tokio::task;

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_module("reqwest::connect", LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    let site = cli.site.clone();
    let desired_chunk_count = cli.chunk_count;

    let mut downloaded_chunk_count = 0;
    let (update_tx, update_rx) = mpsc::channel::<(ChunkIdentifier, Chunk)>();
    let (stats_tx, stats_rx) = mpsc::channel::<PollStats>();
    let (stop_tx, stop_rx) = mpsc::channel::<bool>();

    // Track download attempts per chunk using a Mutex - will be populated from the stats_handle
    let chunk_info = std::sync::Arc::new(Mutex::new(HashMap::<String, ChunkDownloadInfo>::new()));
    let stats_chunk_info = std::sync::Arc::clone(&chunk_info);

    // Task to poll chunks
    task::spawn(async move {
        poll_chunks(&site, update_tx, Some(stats_tx), stop_rx)
            .await
            .expect("Failed to poll chunks");
    });

    // Task to timeout polling at 120 seconds
    let timeout_stop_tx = stop_tx.clone();
    task::spawn(async move {
        tokio::time::sleep(Duration::from_secs(120)).await;

        info!("Timeout reached, stopping...");
        timeout_stop_tx.send(true).unwrap();
    });

    // Task to receive statistics updates to track attempts
    let stats_handle = task::spawn(async move {
        while let Ok(stats) = stats_rx.recv() {
            if let PollStats::NewChunk(new_chunk_stats) = stats {
                // For each new chunk, we'll receive stats before receiving the chunk
                debug!("New chunk download attempts: {}", new_chunk_stats.calls);

                // Store the attempt count in our shared map
                // We'll use a counter to generate a unique key until the update_handle can associate it with a chunk name
                let mut chunk_info_map = stats_chunk_info.lock().unwrap();
                let pending_key = format!("pending_{}", chunk_info_map.len());
                chunk_info_map.insert(
                    pending_key,
                    ChunkDownloadInfo {
                        attempts: new_chunk_stats.calls,
                        processed: false,
                    },
                );
            }
        }
    });

    println!(
        "{:<25} | {:<25} | {:<12} | {:<12} | {:<12} | {:<12} | {:<8}",
        "Chunk",
        "Downloaded",
        "Last Chunk",
        "AWS Upload",
        "First Radial",
        "Last Radial",
        "Attempts"
    );
    println!("{:-<124}", "");

    // Task to receive downloaded chunks
    let update_handle = task::spawn(async move {
        let mut last_chunk_time = None;
        while let Ok((chunk_id, chunk)) = update_rx.recv() {
            let download_time = Utc::now();
            let chunk_name = chunk_id.name().to_string();

            let attempts = {
                let mut chunk_info_map = chunk_info.lock().unwrap();

                // Look for an unprocessed entry (should be the oldest one)
                let pending_key = chunk_info_map
                    .iter()
                    .filter(|(_, info)| !info.processed)
                    .map(|(key, _)| key.clone())
                    .next();

                if let Some(key) = pending_key {
                    // Found a pending entry - update it with the real chunk name
                    if let Some(info) = chunk_info_map.remove(&key) {
                        chunk_info_map.insert(
                            chunk_name.clone(),
                            ChunkDownloadInfo {
                                attempts: info.attempts,
                                processed: true,
                            },
                        );
                        info.attempts
                    } else {
                        // Fallback to 1 if something went wrong
                        1
                    }
                } else {
                    // No pending entries found, use default
                    chunk_info_map.insert(
                        chunk_name.clone(),
                        ChunkDownloadInfo {
                            attempts: 1,
                            processed: true,
                        },
                    );
                    1
                }
            };

            match chunk {
                Chunk::Start(file) => {
                    let records = file.records();
                    debug!(
                        "Volume start chunk with {} records. Header: {:?}",
                        records.len(),
                        file.header()
                    );

                    for record in records {
                        process_record(&chunk_id, record, download_time, last_chunk_time, attempts);
                    }
                }
                Chunk::IntermediateOrEnd(record) => {
                    debug!("Intermediate or end volume chunk.");
                    process_record(&chunk_id, record, download_time, last_chunk_time, attempts);
                }
            }

            last_chunk_time = chunk_id.date_time();

            downloaded_chunk_count += 1;
            if downloaded_chunk_count >= desired_chunk_count {
                info!("Downloaded {} chunks, stopping...", desired_chunk_count);
                stop_tx.send(true).expect("Failed to send stop signal");
                break;
            }
        }
    });

    stats_handle.await.expect("Failed to join handle");
    update_handle.await.expect("Failed to join handle");

    info!("Finished downloading chunks");
    Ok(())
}

#[cfg(all(feature = "aws", feature = "decode"))]
fn process_record(
    chunk_id: &nexrad_data::aws::realtime::ChunkIdentifier,
    mut record: nexrad_data::volume::Record,
    download_time: chrono::DateTime<chrono::Utc>,
    last_chunk_time: Option<chrono::DateTime<chrono::Utc>>,
    attempts: usize,
) {
    debug!("Decoding LDM record...");
    if record.compressed() {
        debug!("Decompressing LDM record...");
        record = record.decompress().expect("Failed to decompress record");
    }

    let messages = match record.messages() {
        Ok(msgs) => msgs,
        Err(e) => {
            warn!("Failed to decode messages: {}", e);
            return;
        }
    };

    let summary = nexrad_decode::summarize::messages(messages.as_slice());

    // Compare chunk_id.date_time() with last_chunk_time, though either could be None
    let time_since_last_chunk = match (chunk_id.date_time(), last_chunk_time) {
        (Some(current), Some(last)) => format!(
            "{:.3}s",
            (current - last).num_milliseconds() as f64 / 1000.0
        ),
        _ => String::from("N/A"),
    };

    // Calculate latencies
    let first_radial_latency = summary
        .earliest_collection_time
        .map(|time| (download_time - time).num_milliseconds() as f64 / 1000.0)
        .unwrap_or(f64::NAN);

    let last_radial_latency = summary
        .latest_collection_time
        .map(|time| (download_time - time).num_milliseconds() as f64 / 1000.0)
        .unwrap_or(f64::NAN);

    let aws_latency = chunk_id
        .date_time()
        .map(|time| (download_time - time).num_milliseconds() as f64 / 1000.0)
        .unwrap_or(f64::NAN);

    // Print concise output in a single line
    println!(
        "{:<25} | {:<25} | {:<12} | {:<12} | {:<12} | {:<12} | {:<8}",
        chunk_id.name(),
        download_time.format("%Y-%m-%d %H:%M:%S%.3f"),
        time_since_last_chunk,
        aws_latency,
        first_radial_latency,
        last_radial_latency,
        attempts
    );
}
