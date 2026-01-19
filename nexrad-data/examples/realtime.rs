#![cfg(all(feature = "aws-polling", not(target_arch = "wasm32")))]

use chrono::{DateTime, Utc};
use clap::Parser;
use env_logger::{Builder, Env};
use log::{debug, info, trace, LevelFilter};
use nexrad_data::result::Result;
use nexrad_data::{
    aws::realtime::{self, poll_chunks, Chunk, ChunkIdentifier, PollStats},
    volume,
};
use nexrad_decode::summarize;
use std::{sync::mpsc, time::Duration};
use tokio::{task, time::sleep};

// Example output from a real-time chunk:
//   Scans from 2025-03-17 01:31:40.449 UTC to 2025-03-17 01:31:44.491 UTC (0.07m)
//   VCPs: VCP35
//   Messages:
//     Msg 1-120: Elevation: #6 (1.36°), Azimuth: 108.2° to 167.7°, Time: 01:31:40.449 to 01:31:44.491 (4.04s)
//       Data types: REF (120), SW (120), VEL (120)

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
    Builder::from_env(Env::default().default_filter_or("debug"))
        .filter_module("reqwest::connect", LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    let site = cli.site.clone();
    let desired_chunk_count = cli.chunk_count;

    let mut downloaded_chunk_count = 0;
    let (update_tx, update_rx) = mpsc::channel::<(ChunkIdentifier, Chunk)>();
    let (stats_tx, stats_rx) = mpsc::channel::<PollStats>();
    let (stop_tx, stop_rx) = mpsc::channel::<bool>();

    // Task to poll chunks
    task::spawn(async move {
        poll_chunks(&site, update_tx, Some(stats_tx), stop_rx)
            .await
            .expect("Failed to poll chunks");
    });

    // Task to timeout polling at 60 seconds
    let timeout_stop_tx = stop_tx.clone();
    task::spawn(async move {
        sleep(Duration::from_secs(60)).await;

        info!("Timeout reached, stopping...");
        timeout_stop_tx.send(true).unwrap();
    });

    // Task to receive statistics updates
    let stats_handle = task::spawn(async move {
        while let Ok(stats) = stats_rx.recv() {
            info!("Polling statistics: {stats:?}");
        }
    });

    // Task to receive downloaded chunks
    let update_handle = task::spawn(async move {
        while let Ok((chunk_id, chunk)) = update_rx.recv() {
            let download_time = Utc::now();

            info!(
                "Downloaded chunk {} from {:?} at {:?} of size {}",
                chunk_id.name(),
                chunk_id.upload_date_time(),
                Utc::now(),
                chunk.data().len()
            );

            match chunk {
                Chunk::Start(file) => {
                    let records = file.records().expect("records");
                    debug!(
                        "Volume start chunk with {} records. Header: {:?}",
                        records.len(),
                        file.header()
                    );

                    records
                        .into_iter()
                        .for_each(|record| decode_record(&chunk_id, record, download_time));
                }
                Chunk::IntermediateOrEnd(record) => {
                    debug!("Intermediate or end volume chunk.");
                    decode_record(&chunk_id, record, download_time);
                }
            }

            downloaded_chunk_count += 1;
            if downloaded_chunk_count >= desired_chunk_count {
                info!("Downloaded 10 chunks, stopping...");
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

fn decode_record(
    chunk_id: &realtime::ChunkIdentifier,
    mut record: volume::Record,
    download_time: DateTime<Utc>,
) {
    debug!("Decoding LDM record...");
    if record.compressed() {
        trace!("Decompressing LDM record...");
        record = record.decompress().expect("Failed to decompress record");
    }

    let messages = record.messages().expect("Failed to decode messages");
    let summary = summarize::messages(messages.as_slice());
    info!("Record summary:\n{summary}");

    info!(
        "Message latency: earliest {:?}, latest {:?}, uploaded: {:?}",
        summary
            .earliest_collection_time
            .map(|time| (download_time - time).num_milliseconds() as f64 / 1000.0),
        summary
            .latest_collection_time
            .map(|time| (download_time - time).num_milliseconds() as f64 / 1000.0),
        chunk_id
            .upload_date_time()
            .map(|time| (download_time - time).num_milliseconds() as f64 / 1000.0),
    );
}
