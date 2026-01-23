#![cfg(all(feature = "aws-polling", not(target_arch = "wasm32")))]

use chrono::{DateTime, Utc};
use clap::Parser;
use env_logger::{Builder, Env};
use futures::StreamExt;
use log::{debug, info, trace, LevelFilter};
use nexrad_data::result::Result;
use nexrad_data::{
    aws::realtime::{self, chunk_stream, Chunk, DownloadedChunk, PollConfig},
    volume,
};
use nexrad_decode::summarize;
use std::pin::pin;
use std::time::Duration;
use tokio::time::timeout;

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
    let desired_chunk_count = cli.chunk_count;

    info!("Starting real-time polling for site {}", cli.site);

    let config = PollConfig::new(&cli.site);
    let stream = chunk_stream(config);
    let mut stream = pin!(stream);

    let mut downloaded_chunk_count = 0;

    // Stream chunks with a 60 second overall timeout
    let result = timeout(Duration::from_secs(60), async {
        while let Some(result) = stream.next().await {
            match result {
                Ok(downloaded) => {
                    let download_time = Utc::now();
                    process_chunk(&downloaded, download_time);

                    downloaded_chunk_count += 1;
                    if downloaded_chunk_count >= desired_chunk_count {
                        info!("Downloaded {} chunks, stopping...", desired_chunk_count);
                        break;
                    }
                }
                Err(e) => {
                    info!("Error downloading chunk: {:?}", e);
                }
            }
        }
    })
    .await;

    match result {
        Ok(()) => info!("Finished downloading chunks"),
        Err(_) => info!("Timeout reached, stopping..."),
    }

    Ok(())
}

fn process_chunk(downloaded: &DownloadedChunk, download_time: DateTime<Utc>) {
    let chunk_id = &downloaded.identifier;
    let chunk = &downloaded.chunk;

    info!(
        "Downloaded chunk {} from {:?} at {:?} of size {} (attempts: {})",
        chunk_id.name(),
        chunk_id.upload_date_time(),
        download_time,
        chunk.data().len(),
        downloaded.attempts
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
                .for_each(|record| decode_record(chunk_id, record, download_time));
        }
        Chunk::IntermediateOrEnd(record) => {
            debug!("Intermediate or end volume chunk.");
            decode_record(chunk_id, record.clone(), download_time);
        }
    }
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
