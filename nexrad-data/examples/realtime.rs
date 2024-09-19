use clap::Parser;
use log::{debug, info, trace, LevelFilter};

// Example output from a real-time chunk:
//
//     MessageSummary {
//         volume_coverage_patterns: {
//             VCP35,
//         },
//         message_types: [
//             "RDADigitalRadarDataGenericFormat: 120",
//         ],
//         scans: [
//             ScanSummary {
//                 start_time: Some(
//                     2024-09-19T03:24:59.799Z,
//                 ),
//                 end_time: Some(
//                     2024-09-19T03:25:11.629Z,
//                 ),
//                 elevation: 3,
//                 start_azimuth: 273.25195,
//                 end_azimuth: 332.75116,
//                 data_types: [
//                     "Reflectivity: 120",
//                     "Differential Phase: 120",
//                     "Specific Differential Phase: 120",
//                     "Differential Reflectivity: 120",
//                     "Correlation Coefficient: 120",
//                 ],
//             },
//         ],
//         earliest_collection_time: Some(
//             2024-09-19T03:24:59.799Z,
//         ),
//         latest_collection_time: Some(
//             2024-09-19T03:25:11.629Z,
//         ),
//     }

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

#[cfg(all(feature = "aws", feature = "decode"))]
#[tokio::main]
async fn main() -> nexrad_data::result::Result<()> {
    use chrono::Utc;
    use nexrad_data::aws::realtime::Chunk;
    use nexrad_data::aws::realtime::{poll_chunks, ChunkIdentifier, PollStats};
    use std::sync::mpsc;
    use std::time::Duration;
    use tokio::task;

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
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
        tokio::time::sleep(Duration::from_secs(60)).await;

        info!("Timeout reached, stopping...");
        timeout_stop_tx.send(true).unwrap();
    });

    // Task to receive statistics updates
    let stats_handle = task::spawn(async move {
        while let Ok(stats) = stats_rx.recv() {
            info!("Polling statistics: {:?}", stats);
        }
    });

    // Task to receive downloaded chunks
    let update_handle = task::spawn(async move {
        while let Ok((chunk_id, chunk)) = update_rx.recv() {
            let download_time = Utc::now();

            info!(
                "Downloaded chunk {} from {:?} at {:?} of size {}",
                chunk_id.name(),
                chunk_id.date_time(),
                Utc::now(),
                chunk.data().len()
            );

            match chunk {
                Chunk::Start(file) => {
                    let records = file.records();
                    debug!(
                        "Volume start chunk with {} records. Header: {:?}",
                        records.len(),
                        file.header()
                    );

                    records
                        .into_iter()
                        .for_each(|record| decode_record(record, download_time));
                }
                Chunk::IntermediateOrEnd(record) => {
                    debug!("Intermediate or end volume chunk.");
                    decode_record(record, download_time);
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

#[cfg(all(feature = "aws", feature = "decode"))]
fn decode_record(
    mut record: nexrad_data::volume::Record,
    download_time: chrono::DateTime<chrono::Utc>,
) {
    debug!("Decoding LDM record...");
    if record.compressed() {
        trace!("Decompressing LDM record...");
        record = record.decompress().expect("Failed to decompress record");
    }

    let messages = record.messages().expect("Failed to decode messages");
    let summary = nexrad_decode::summarize::messages(messages.as_slice());
    info!("Record summary:\n{:#?}", summary);

    info!(
        "Message latency: {:?}",
        summary
            .earliest_collection_time
            .map(|time| download_time - time),
    );
}
