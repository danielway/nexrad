#![cfg(all(feature = "aws"))]

use chrono::{NaiveDate, NaiveTime};
use nexrad_data::archive::Identifier;
use nexrad_data::aws::archive::{download_file, list_files};
use nexrad_data::result::Result;
use std::env;
use std::fs::{create_dir, File};
use std::io::Write;
use std::path::Path;

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
    let files = list_files(site, &date).await?;

    if files.is_empty() {
        println!("No files found for the specified date/site to download.");
        return Ok(());
    }

    println!("Found {} files.", files.len());

    let start_index = get_nearest_file_index(&files, start_time);
    println!(
        "Nearest file to start of {:?} is {:?}.",
        start_time,
        files[start_index].name()
    );

    let stop_index = get_nearest_file_index(&files, stop_time);
    println!(
        "Nearest file to stop of {:?} is {:?}.",
        stop_time,
        files[stop_index].name()
    );

    println!("Downloading {} files...", stop_index - start_index + 1);

    for file in files
        .iter()
        .skip(start_index)
        .take(stop_index - start_index + 1)
    {
        println!("Downloading file \"{}\"...", file.name());
        let downloaded_file = download_file(file).await?;

        println!("Data file size (bytes): {}", downloaded_file.len());

        if !Path::new("downloads").exists() {
            println!("Creating downloads directory...");
            create_dir("downloads").expect("create downloads directory");
        }

        println!("Writing file to disk as: {}", file.name());
        let mut file = File::create(format!("downloads/{}", file.name())).expect("create file");
        file.write_all(downloaded_file.as_slice())
            .expect("write file");
    }

    println!("Downloaded {} files.", stop_index - start_index + 1);

    Ok(())
}

/// Returns the index of the file with the nearest time to the provided start time.
fn get_nearest_file_index(files: &Vec<Identifier>, start_time: NaiveTime) -> usize {
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
