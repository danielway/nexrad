//! examples/download
//!
//! This example downloads a data file for some date/site and prints its size.
//!
//! Usage: cargo run --example download -- [site] [date] [start_time] [stop_time]
//!

#![cfg(all(feature = "download", feature = "decompress"))]

use chrono::{NaiveDate, NaiveTime};
use std::env;
use std::fs::{create_dir, File};
use std::io::Write;
use std::path::Path;

use nexrad::download::{download_file, list_files};
use nexrad::file::FileMetadata;
use nexrad::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut site = "KDMX";
    if args.len() > 1 {
        site = &args[1];
    }

    let mut date = NaiveDate::from_ymd_opt(2022, 3, 5).expect("is valid date");
    if args.len() > 2 {
        let date_str = &args[2];
        date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").expect("is valid date");
    }

    let mut start_time = NaiveTime::from_hms_opt(23, 30, 0).expect("is valid time");
    if args.len() > 3 {
        let time_str = &args[3];
        start_time = NaiveTime::parse_from_str(time_str, "%H:%M").expect("start is valid time");
    }

    let mut stop_time = start_time;
    if args.len() > 4 {
        let time_str = &args[4];
        stop_time = NaiveTime::parse_from_str(time_str, "%H:%M").expect("stop is valid time");
    }

    println!("Listing files for {} on {}...", site, date);
    let metas = list_files(site, &date).await?;

    if metas.is_empty() {
        println!("No files found for the specified date/site to download.");
        return Ok(());
    }

    println!("Found {} files.", metas.len());

    let start_index = get_nearest_metadata_index(&metas, start_time);
    println!(
        "Nearest file to start of {:?} is {:?}.",
        start_time,
        metas[start_index].identifier()
    );

    let stop_index = get_nearest_metadata_index(&metas, stop_time);
    println!(
        "Nearest file to stop of {:?} is {:?}.",
        stop_time,
        metas[stop_index].identifier()
    );

    println!("Downloading {} files...", stop_index - start_index + 1);

    for meta in metas
        .iter()
        .skip(start_index)
        .take(stop_index - start_index + 1)
    {
        println!("Downloading file \"{}\"...", meta.identifier());
        let downloaded_file = download_file(meta).await?;

        println!("Data file size (bytes): {}", downloaded_file.len());

        if !Path::new("downloads").exists() {
            println!("Creating downloads directory...");
            create_dir("downloads").expect("create downloads directory");
        }

        println!("Writing file to disk as: {}", meta.identifier());
        let mut file =
            File::create(format!("downloads/{}", meta.identifier())).expect("create file");
        file.write_all(downloaded_file.as_slice())
            .expect("write file");
    }

    println!("Downloaded {} files.", stop_index - start_index + 1);

    Ok(())
}

/// Returns the index of the metadata with the nearest time to the provided start time.
fn get_nearest_metadata_index(metas: &Vec<FileMetadata>, start_time: NaiveTime) -> usize {
    let first_metadata_time = get_metadata_time(metas.first().expect("found at least one meta"));
    let mut min_diff = first_metadata_time
        .signed_duration_since(start_time)
        .num_seconds()
        .abs();
    let mut min_index = 0;

    for (index, metadata) in metas.iter().skip(1).enumerate() {
        let metadata_time = get_metadata_time(metadata);
        let diff = metadata_time
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

/// Returns the time from the metadata identifier.
fn get_metadata_time(metadata: &FileMetadata) -> NaiveTime {
    let identifier_parts = metadata.identifier().split('_');
    let identifier_time = identifier_parts.collect::<Vec<_>>()[1];
    NaiveTime::parse_from_str(identifier_time, "%H%M%S").expect("is valid time")
}
