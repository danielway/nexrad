#![cfg(feature = "aws")]

use chrono::{NaiveDate, NaiveTime};
use clap::Parser;
use env_logger::{Builder, Env};
use log::{debug, info, trace, warn, LevelFilter};
use nexrad_data::aws::archive::{self, download_file, list_files};
use nexrad_data::result::Result;
use nexrad_data::volume::File;
use nexrad_decode::messages::digital_radar_data::ScaledMomentValue;
use nexrad_decode::messages::MessageContents;
use std::fs::create_dir;
use std::io::Read;
use std::io::Write;
use std::path::Path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Site identifier (e.g., KDMX)
    #[arg(default_value = "KDMX")]
    site: String,

    /// Date in YYYY-MM-DD format
    #[arg(default_value = "2022-03-05")]
    date: String,

    /// Start time in HH:MM format
    #[arg(default_value = "23:30")]
    start_time: String,

    /// Stop time in HH:MM format
    #[arg(default_value = "23:30")]
    stop_time: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("debug"))
        .filter_module("reqwest::connect", LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    let site = &cli.site;
    let date = NaiveDate::parse_from_str(&cli.date, "%Y-%m-%d").expect("is valid date");
    let start_time =
        NaiveTime::parse_from_str(&cli.start_time, "%H:%M").expect("start is valid time");
    let stop_time = NaiveTime::parse_from_str(&cli.stop_time, "%H:%M").expect("stop is valid time");

    info!("Listing files for {site} on {date}...");
    let file_ids = list_files(site, &date).await?;

    if file_ids.is_empty() {
        warn!("No files found for the specified date/site to download.");
        return Ok(());
    }

    debug!("Found {} files.", file_ids.len());

    let start_index = get_nearest_file_index(&file_ids, start_time);
    debug!(
        "Nearest file to start of {:?} is {:?}.",
        start_time,
        file_ids[start_index].name()
    );

    let stop_index = get_nearest_file_index(&file_ids, stop_time);
    debug!(
        "Nearest file to stop of {:?} is {:?}.",
        stop_time,
        file_ids[stop_index].name()
    );

    debug!("Downloading {} files...", stop_index - start_index + 1);
    for file_id in file_ids
        .iter()
        .skip(start_index)
        .take(stop_index - start_index + 1)
    {
        if file_id.name().ends_with("_MDM") {
            debug!("Skipping MDM file: {}", file_id.name());
            continue;
        }

        let file = if Path::new(&format!("downloads/{}", file_id.name())).exists() {
            debug!("File \"{}\" already downloaded.", file_id.name());
            let mut file =
                std::fs::File::open(format!("downloads/{}", file_id.name())).expect("open file");

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).expect("read file");

            File::new(buffer)
        } else {
            debug!("Downloading file \"{}\"...", file_id.name());
            let file = download_file(file_id.clone()).await?;

            if !Path::new("downloads").exists() {
                trace!("Creating downloads directory...");
                create_dir("downloads").expect("create downloads directory");
            }

            trace!("Writing file to disk as: {}", file_id.name());
            let mut downloaded_file =
                std::fs::File::create(format!("downloads/{}", file_id.name()))
                    .expect("create file");

            downloaded_file.write_all(file.data()).expect("write file");

            file
        };

        trace!("Data file size (bytes): {}", file.data().len());

        let records = file.records()?;
        debug!(
            "Volume with {} records. Header: {:?}",
            records.len(),
            file.header()
        );

        debug!("Decoding {} records...", records.len());
        for (record_index, mut record) in records.into_iter().enumerate() {
            debug!("Decoding record {}", record_index + 1);
            if record.compressed() {
                trace!("Decompressing LDM record...");
                record = record.decompress().expect("Failed to decompress record");
            }

            let messages = record.messages()?;
            info!(
                "Decoded {} messages in record {}:",
                messages.len(),
                record_index + 1
            );

            for (message_index, message) in messages.iter().enumerate() {
                if let MessageContents::DigitalRadarData(data) = message.contents() {
                    if let Some(block) = data.reflectivity_data_block() {
                        info!(
                            "Message {} at {:?}, reflectivity:",
                            message_index,
                            message.header().date_time(),
                        );
                        info!(
                            "  {}",
                            scaled_values_to_ascii(&block.decoded_values()[..100])
                        );
                    }
                } else {
                    info!(
                        "Message {} at {:?}: type {:?}",
                        message_index,
                        message.header().date_time(),
                        message.header().message_type()
                    );
                }
            }
        }

        info!("All records decoded.");
    }

    Ok(())
}

/// Converts a slice of ScaledMomentValue to a visual ASCII string representation.
/// Uses characters ordered by visual density to represent radar reflectivity values.
fn scaled_values_to_ascii(values: &[ScaledMomentValue]) -> String {
    // Characters ordered by increasing visual density
    const CHARS: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

    // dBZ range for radar reflectivity (typical range)
    const MIN_DBZ: f32 = -30.0;
    const MAX_DBZ: f32 = 75.0;

    values
        .iter()
        .map(|v| match v {
            ScaledMomentValue::Value(val) => {
                // Normalize to 0.0-1.0 range, then map to character index
                let normalized = ((val - MIN_DBZ) / (MAX_DBZ - MIN_DBZ)).clamp(0.0, 1.0);
                let index = (normalized * (CHARS.len() - 1) as f32) as usize;
                CHARS[index]
            }
            ScaledMomentValue::BelowThreshold => ' ',
            ScaledMomentValue::RangeFolded => '~',
            ScaledMomentValue::CfpStatus(_) => '!',
        })
        .collect()
}

/// Returns the index of the file with the nearest time to the provided start time.
fn get_nearest_file_index(files: &[archive::Identifier], start_time: chrono::NaiveTime) -> usize {
    let first_file = files.first().expect("find at least one file");
    let first_file_time = first_file
        .date_time()
        .expect("file has valid date time")
        .time();
    let mut min_diff = first_file_time
        .signed_duration_since(start_time)
        .num_seconds()
        .abs();
    let mut min_index = 0;

    for (index, file) in files.iter().skip(1).enumerate() {
        let file_time = file.date_time().expect("file has valid date time").time();
        let diff = file_time
            .signed_duration_since(start_time)
            .num_seconds()
            .abs();

        if diff < min_diff {
            min_diff = diff;
            min_index = index;
        }
    }

    min_index
}
