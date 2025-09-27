#![cfg(all(feature = "aws", feature = "decode"))]

use chrono::{NaiveDate, NaiveTime};
use clap::Parser;
use env_logger::{Builder, Env};
use log::{debug, info, trace, warn, LevelFilter};
use nexrad_data::aws::archive::{self, download_file, list_files};
use nexrad_data::result::Result;
use nexrad_data::volume::File;
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

            downloaded_file
                .write_all(file.data().as_slice())
                .expect("write file");

            file
        };

        trace!("Data file size (bytes): {}", file.data().len());

        let records = file.records();
        debug!(
            "Volume with {} records. Header: {:?}",
            records.len(),
            file.header()
        );

        debug!("Decoding {} records...", records.len());
        let mut messages = Vec::new();
        for mut record in records {
            if record.compressed() {
                trace!("Decompressing LDM record...");
                record = record.decompress().expect("Failed to decompress record");
            }

            messages.extend(record.messages()?.iter().cloned());
        }

        let summary = nexrad_decode::summarize::messages(messages.as_slice());
        info!("Volume summary:\n{summary}");
    }

    Ok(())
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
