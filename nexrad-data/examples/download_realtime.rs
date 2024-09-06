//! This example downloads real-time data (streaming via polling which attempts to only query when
//! data is expected to be available) from the NEXRAD AWS S3 real-time data bucket. It will decode
//! the downloaded chunks and provide information about the messages and scans contained.
//!
//! Here is a sample of output. In it you can see two chunks downloaded and inspected. Both are
//! downloaded within a few seconds of being uploaded to S3, but they are downloaded approximately
//! 15 seconds from when the RDA generated the message. This means real-time data will always be
//! approximately 15 seconds old from when the radar unit collected the data.
//!
//! [2024-09-06T14:59:19Z DEBUG nexrad_data::aws::s3::download_object] Downloading object key "KDMX/378/20240906-145852-003-I" from bucket "unidata-nexrad-level2-chunks"
//! [2024-09-06T14:59:19Z INFO  download_realtime] Polling statistics: NewChunk(NewChunkStats { calls: 1, latency: Some(2.354861s) })
//! [2024-09-06T14:59:19Z INFO  download_realtime] Downloaded chunk 20240906-145852-003-I from Some(2024-09-06T14:59:17Z) at 2024-09-06T14:59:19.354952Z of size 253146
//! [2024-09-06T14:59:19Z DEBUG download_realtime] Intermediate or end volume chunk.
//! [2024-09-06T14:59:19Z DEBUG download_realtime] Message latency: Some(TimeDelta { secs: 14, nanos: 867949000 }), Coverage pattern: Some(35)
//! [2024-09-06T14:59:19Z DEBUG download_realtime] Message type RDADigitalRadarDataGenericFormat has 120 messages
//! [2024-09-06T14:59:19Z INFO  download_realtime] ScanData { azimuth: 291-351, elevation: 1, data_types: {"Differential Phase": 119, "Specific Differential Phase": 119, "Correlation Coefficient": 119, "Differential Reflectivity": 119, "Reflectivity": 119} }
//! [2024-09-06T14:59:29Z DEBUG nexrad_data::aws::s3::download_object] Downloading object key "KDMX/378/20240906-145852-004-I" from bucket "unidata-nexrad-level2-chunks"
//! [2024-09-06T14:59:29Z DEBUG nexrad_data::aws::s3::download_object] Downloading object key "KDMX/378/20240906-145852-004-I" from bucket "unidata-nexrad-level2-chunks"
//! [2024-09-06T14:59:30Z DEBUG nexrad_data::aws::s3::download_object] Downloading object key "KDMX/378/20240906-145852-004-I" from bucket "unidata-nexrad-level2-chunks"
//! [2024-09-06T14:59:31Z INFO  download_realtime] Polling statistics: NewChunk(NewChunkStats { calls: 3, latency: Some(271.942ms) })
//! [2024-09-06T14:59:31Z INFO  download_realtime] Downloaded chunk 20240906-145852-004-I from Some(2024-09-06T14:59:31Z) at 2024-09-06T14:59:31.272025Z of size 268005
//! [2024-09-06T14:59:31Z DEBUG download_realtime] Intermediate or end volume chunk.
//! [2024-09-06T14:59:31Z DEBUG download_realtime] Message latency: Some(TimeDelta { secs: 14, nanos: 841023000 }), Coverage pattern: Some(35)
//! [2024-09-06T14:59:31Z DEBUG download_realtime] Message type RDADigitalRadarDataGenericFormat has 120 messages
//! [2024-09-06T14:59:31Z INFO  download_realtime] ScanData { azimuth: 351-51, elevation: 1, data_types: {"Correlation Coefficient": 119, "Differential Phase": 119, "Reflectivity": 119, "Specific Differential Phase": 119, "Differential Reflectivity": 119} }
//!

use chrono::{DateTime, Utc};
use log::{debug, info, trace, LevelFilter};
use std::collections::HashMap;
use std::fmt::Display;
use std::io::{Cursor, Seek, SeekFrom};
use std::sync::mpsc;
use std::time::Duration;
use tokio::task;

use nexrad_data::volume::Record;
use nexrad_decode::messages::digital_radar_data::decode_digital_radar_data;
use nexrad_decode::messages::message_header::MessageHeader;
use nexrad_decode::messages::{decode_message_header, MessageType};

#[cfg(feature = "aws")]
use nexrad_data::aws::realtime::{poll_chunks, Chunk, ChunkIdentifier, PollStats};

#[cfg(not(feature = "aws"))]
fn main() {
    info!("This example requires the \"aws\" feature to be enabled.");
}

#[cfg(feature = "aws")]
#[tokio::main]
async fn main() -> nexrad_data::result::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .filter_module("reqwest::connect", LevelFilter::Info)
        .init();

    let mut downloaded_chunks = 0;

    let (update_tx, update_rx) = mpsc::channel::<(ChunkIdentifier, Chunk)>();
    let (stats_tx, stats_rx) = mpsc::channel::<PollStats>();
    let (stop_tx, stop_rx) = mpsc::channel::<bool>();

    task::spawn(async move {
        poll_chunks("KDMX", update_tx, Some(stats_tx), stop_rx)
            .await
            .expect("Failed to poll chunks");
    });

    let timeout_stop_tx = stop_tx.clone();
    task::spawn(async move {
        tokio::time::sleep(Duration::from_secs(60)).await;

        info!("Timeout reached, stopping...");
        timeout_stop_tx.send(true).unwrap();
    });

    let stats_handle = task::spawn(async move {
        while let Ok(stats) = stats_rx.recv() {
            info!("Polling statistics: {:?}", stats);
        }
    });

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

                    // todo: something is off here
                    records
                        .into_iter()
                        .for_each(|record| decode_record(record, download_time));
                }
                Chunk::IntermediateOrEnd(record) => {
                    debug!("Intermediate or end volume chunk.");
                    decode_record(record, download_time);
                }
            }

            downloaded_chunks += 1;
            if downloaded_chunks >= 10 {
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

#[cfg(feature = "aws")]
fn decode_record(mut record: Record, download_time: DateTime<Utc>) {
    if record.compressed() {
        trace!("Decompressing LDM record...");
        record = record.decompress().expect("Failed to decompress record");
    } else {
        trace!("Decompressed LDM record");
    }

    let mut message_type_counts = HashMap::new();

    let mut all_scans = Vec::new();
    let mut first_message_time = None;
    let mut coverage_pattern = None;
    let mut scan_data: Option<ScanData> = None;

    let mut reader = Cursor::new(record.data());
    while reader.position() < reader.get_ref().len() as u64 {
        let message_header =
            decode_message_header(&mut reader).expect("Failed to decode message header");

        if first_message_time.is_none() {
            first_message_time = message_header.date_time();
        }

        let message_type = message_header.message_type();
        let count = message_type_counts.get(&message_type).unwrap_or(&0) + 1;
        message_type_counts.insert(message_type, count);

        if message_header.message_type() == MessageType::RDADigitalRadarDataGenericFormat {
            let m31 = decode_digital_radar_data(&mut reader).expect("Failed to decode M31 message");

            if coverage_pattern.is_none() {
                coverage_pattern = Some(
                    m31.volume_data_block
                        .expect("No volume data block")
                        .volume_coverage_pattern_number,
                );
            }

            if let Some(current_scan_data) = scan_data.as_mut() {
                if current_scan_data.elevation == m31.header.elevation_number {
                    current_scan_data.end_azimuth = m31.header.azimuth_angle;

                    let mut increment_count = |data_type: &str| {
                        let count = current_scan_data.data_types.get(data_type).unwrap_or(&0) + 1;
                        current_scan_data
                            .data_types
                            .insert(data_type.to_string(), count);
                    };

                    if m31.reflectivity_data_block.is_some() {
                        increment_count("Reflectivity");
                    }
                    if m31.velocity_data_block.is_some() {
                        increment_count("Velocity");
                    }
                    if m31.spectrum_width_data_block.is_some() {
                        increment_count("Spectrum Width");
                    }
                    if m31.differential_reflectivity_data_block.is_some() {
                        increment_count("Differential Reflectivity");
                    }
                    if m31.differential_phase_data_block.is_some() {
                        increment_count("Differential Phase");
                    }
                    if m31.correlation_coefficient_data_block.is_some() {
                        increment_count("Correlation Coefficient");
                    }
                    if m31.specific_diff_phase_data_block.is_some() {
                        increment_count("Specific Differential Phase");
                    }
                } else {
                    all_scans.push(format!("{}", current_scan_data));
                    scan_data = None;
                }
            }

            if scan_data.is_none() {
                scan_data = Some(ScanData {
                    start_azimuth: m31.header.azimuth_angle,
                    end_azimuth: m31.header.azimuth_angle,
                    elevation: m31.header.elevation_number,
                    data_types: HashMap::new(),
                });
            }
        } else {
            if let Some(scan_data) = scan_data.take() {
                all_scans.push(format!("{}", scan_data));
            }

            // Non-M31 messages are 2432 bytes long, including the header
            reader
                .seek(SeekFrom::Current(2432 - size_of::<MessageHeader>() as i64))
                .unwrap();
        }
    }

    if let Some(scan_info) = scan_data.take() {
        all_scans.push(format!("{}", scan_info));
    }

    debug!(
        "Message latency: {:?}, Coverage pattern: {:?}",
        first_message_time.map(|time| download_time - time),
        coverage_pattern
    );

    for (message_type, count) in message_type_counts {
        debug!("Message type {:?} has {} messages", message_type, count);

        if message_type == MessageType::RDADigitalRadarDataGenericFormat {
            info!("{}", all_scans.join(", "));
        }
    }
}

struct ScanData {
    start_azimuth: f32,
    end_azimuth: f32,
    elevation: u8,
    data_types: HashMap<String, usize>,
}

impl Display for ScanData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ScanData {{ azimuth: {:.0}-{:.0}, elevation: {}, data_types: {:?} }}",
            self.start_azimuth, self.end_azimuth, self.elevation, self.data_types
        )
    }
}
