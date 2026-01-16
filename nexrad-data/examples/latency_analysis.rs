#![cfg(feature = "aws")]

use chrono::{DateTime, SubsecRound, Utc};
use clap::Parser;
use env_logger::{Builder, Env};
use log::{debug, info, warn, LevelFilter};
use nexrad_data::result::Result;
use nexrad_data::{aws::realtime, volume};
use nexrad_decode::summarize;
use tokio::time::sleep;

// Example designed to provide concise latency analysis for NEXRAD data chunks
// Output format (single line per chunk):
// Chunk: <name> | Downloaded: <time> | AWS Latency: <value>s | First Radial Latency: <value>s | Last Radial Latency: <value>s | Attempts: <count>

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

#[tokio::main]
async fn main() -> Result<()> {
    use chrono::Utc;
    use nexrad_data::aws::realtime::Chunk;
    use nexrad_data::aws::realtime::{poll_chunks, ChunkIdentifier, PollStats};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{mpsc, Arc, Mutex};
    use std::time::Duration;
    use tokio::task;

    Builder::from_env(Env::default().default_filter_or("info"))
        .filter_module("reqwest::connect", LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    let site = cli.site.clone();
    let desired_chunk_count = cli.chunk_count;

    let mut downloaded_chunk_count = 0;
    let (update_tx, update_rx) = mpsc::channel::<(ChunkIdentifier, Chunk)>();
    let (stats_tx, stats_rx) = mpsc::channel::<PollStats>();
    let (stop_tx, stop_rx) = mpsc::channel::<bool>();

    // Pass download attempts and timing to the update handle
    let attempts = Arc::new(AtomicUsize::new(0));
    let download_time = Arc::new(Mutex::new(None::<chrono::DateTime<chrono::Utc>>));

    // Task to poll chunks
    task::spawn(async move {
        poll_chunks(&site, update_tx, Some(stats_tx), stop_rx)
            .await
            .expect("Failed to poll chunks");
    });

    // Task to timeout polling at 5 minutes
    let timeout_stop_tx = stop_tx.clone();
    task::spawn(async move {
        sleep(Duration::from_secs(300)).await;

        info!("Timeout reached, stopping...");
        timeout_stop_tx
            .send(true)
            .expect("Failed to send stop signal");
    });

    // Task to receive statistics updates to track attempts
    let attempts_clone = attempts.clone();
    let download_time_clone = download_time.clone();
    let stats_handle = task::spawn(async move {
        while let Ok(stats) = stats_rx.recv() {
            match stats {
                PollStats::NewChunk(new_chunk_stats) => {
                    debug!(
                        "New chunk download: attempts={}, download_time={:?}, upload_time={:?}",
                        new_chunk_stats.calls,
                        new_chunk_stats.download_time,
                        new_chunk_stats.upload_time
                    );

                    attempts_clone.fetch_add(new_chunk_stats.calls, Ordering::SeqCst);

                    let mut download_time_guard = download_time_clone
                        .lock()
                        .expect("Failed to lock download time");
                    if let Some(time) = new_chunk_stats.download_time {
                        *download_time_guard = Some(time);
                    } else {
                        *download_time_guard = None;
                    }
                }
                PollStats::NewVolumeCalls(new_volume_stats) => {
                    debug!("New volume found: attempts={new_volume_stats}");
                    attempts_clone.fetch_add(new_volume_stats, Ordering::SeqCst);
                }
                PollStats::ChunkTimings(chunk_timings) => {
                    info!("Chunk Timing Statistics:");
                    info!("{:-<100}", "");
                    info!(
                        "{:<15} | {:<20} | {:<20} | {:<15} | {:<15}",
                        "Chunk Type",
                        "Waveform Type",
                        "Channel Config",
                        "Avg Duration",
                        "Avg Attempts"
                    );
                    info!("{:-<100}", "");

                    for (characteristics, avg_duration, avg_attempts) in
                        chunk_timings.get_statistics()
                    {
                        let duration_str = avg_duration.map_or("N/A".to_string(), |d| {
                            format!("{:.2}s", d.num_milliseconds() as f64 / 1000.0)
                        });

                        let attempts_str =
                            avg_attempts.map_or("N/A".to_string(), |a| format!("{a:.2}"));

                        info!(
                            "{:<15} | {:<20} | {:<20} | {:<15} | {:<15}",
                            format!("{:?}", characteristics.chunk_type),
                            format!("{:?}", characteristics.waveform_type),
                            format!("{:?}", characteristics.channel_configuration),
                            duration_str,
                            attempts_str
                        );
                    }

                    info!("{:-<100}", "");
                }
                _ => {}
            }
        }
    });

    println!(
        "{:<25} | {:<25} | {:<13} | {:<15} {:<29} | {:<8}",
        "", "", "Time Since", "", "Latency Since", ""
    );
    println!(
        "{:<25} | {:<25} | {:<13} | {:<13} | {:<13} | {:<13} | {:<8}",
        "Chunk",
        "Downloaded",
        "Last Chunk",
        "AWS Upload",
        "First Radial",
        "Last Radial",
        "Attempts"
    );
    println!("{:-<128}", "");

    // Task to receive downloaded chunks
    let update_handle = task::spawn(async move {
        let mut last_chunk_time = None;
        while let Ok((chunk_id, chunk)) = update_rx.recv() {
            let download_time = {
                match *download_time.lock().expect("Failed to lock download time") {
                    Some(time) => time,
                    None => {
                        warn!("No download time available, using current time");
                        Utc::now()
                    }
                }
            };

            let chunk_attempts = attempts.load(Ordering::SeqCst);
            attempts.store(0, Ordering::SeqCst);

            match chunk {
                Chunk::Start(file) => {
                    let records = file.records().expect("records");
                    debug!(
                        "Volume start chunk with {} records. Header: {:?}",
                        records.len(),
                        file.header()
                    );

                    for record in records {
                        process_record(
                            &chunk_id,
                            record,
                            download_time,
                            last_chunk_time,
                            chunk_attempts,
                        );
                    }
                }
                Chunk::IntermediateOrEnd(record) => {
                    debug!("Intermediate or end volume chunk.");
                    process_record(
                        &chunk_id,
                        record,
                        download_time,
                        last_chunk_time,
                        chunk_attempts,
                    );
                }
            }

            last_chunk_time = chunk_id.upload_date_time();

            downloaded_chunk_count += 1;
            if downloaded_chunk_count >= desired_chunk_count {
                info!("Downloaded {desired_chunk_count} chunks, stopping...");
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

fn process_record(
    chunk_id: &realtime::ChunkIdentifier,
    mut record: volume::Record,
    download_time: DateTime<Utc>,
    last_chunk_time: Option<DateTime<Utc>>,
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
            warn!("Failed to decode messages: {e}");
            return;
        }
    };

    let summary = summarize::messages(messages.as_slice());

    // Calculate latencies
    let first_radial_latency = summary
        .earliest_collection_time
        .map(|time| (download_time - time).num_milliseconds() as f64 / 1000.0)
        .unwrap_or(f64::NAN);

    let last_radial_latency = summary
        .latest_collection_time
        .map(|time| (download_time - time).num_milliseconds() as f64 / 1000.0)
        .unwrap_or(f64::NAN);

    // AWS rounds to the second for object modified times
    let rounded_download_time = download_time.round_subsecs(0);

    let aws_latency = chunk_id
        .upload_date_time()
        .map(|time| {
            if rounded_download_time < time {
                warn!("Download time is before S3 modified time: download={download_time}, rounded download={rounded_download_time}, s3={time}");
            }

            (rounded_download_time - time).num_milliseconds() as f64 / 1000.0
        })
        .unwrap_or(f64::NAN);

    // Compare chunk_id.date_time() with last_chunk_time, though either could be None
    let time_since_last_chunk = match (chunk_id.upload_date_time(), last_chunk_time) {
        (Some(current), Some(last)) => {
            format!("{}", (current - last).num_milliseconds() as f64 / 1000.0)
        }
        _ => String::from("N/A"),
    };

    // Print concise output in a single line
    println!(
        "{:<25} | {:<25} | {:<12}s | {:<12}s | {:<12}s | {:<12}s | {:<8}",
        format!("{}/{}", chunk_id.volume().as_number(), chunk_id.name()),
        download_time.format("%Y-%m-%d %H:%M:%S%.3f"),
        time_since_last_chunk,
        aws_latency,
        first_radial_latency,
        last_radial_latency,
        attempts
    );
}
