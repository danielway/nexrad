//! examples/download
//!
//! This example downloads a data file for some date/site and prints its size.
//!
//! Usage: cargo run --example download -- [site] [date] [time]
//!

#![cfg(all(feature = "download", feature = "decompress"))]

use chrono::{NaiveDate, NaiveTime};
use std::env;
use std::fs::{create_dir, File};
use std::io::Write;
use std::path::Path;

use nexrad::download::{download_file, list_files};
use nexrad::file::is_compressed;
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

    let mut requested_time = NaiveTime::from_hms_opt(23, 30, 0).expect("is valid time");
    if args.len() > 3 {
        let time_str = &args[3];
        requested_time = NaiveTime::parse_from_str(time_str, "%H:%M").expect("is valid time");
    }

    println!("Listing files for {} on {}...", site, date);
    let metas = list_files(site, &date).await?;

    if metas.is_empty() {
        println!("No files found for the specified date/site to download.");
        return Ok(());
    }

    println!("Found {} files.", metas.len());

    let mut meta = metas.first().expect("found at least one meta");

    let mut min_diff = std::i64::MAX;
    for m in metas.iter() {
        let identifier_parts = m.identifier().split('_');
        let identifier_time = identifier_parts.collect::<Vec<_>>()[1];
        let identifier_time =
            NaiveTime::parse_from_str(identifier_time, "%H%M%S").expect("is valid time");

        let diff = (identifier_time.signed_duration_since(requested_time))
            .num_seconds()
            .abs();

        if diff < min_diff {
            min_diff = diff;
            meta = m;
        }
    }

    println!(
        "Nearest file to {:?} is {:?}.",
        requested_time,
        meta.identifier()
    );

    println!("Downloading file \"{}\"...", meta.identifier());
    let downloaded_file = download_file(meta).await?;

    println!("Data file size (bytes): {}", downloaded_file.len());

    let is_compressed = is_compressed(downloaded_file.as_slice());
    println!("File data is compressed: {}", is_compressed);

    if !Path::new("downloads").exists() {
        create_dir("downloads").expect("create downloads directory");
    }

    println!("Writing file to disk as: {}", meta.identifier());
    let mut file = File::create(format!("downloads/{}", meta.identifier())).expect("create file");
    file.write_all(downloaded_file.as_slice())
        .expect("write file");

    Ok(())
}
