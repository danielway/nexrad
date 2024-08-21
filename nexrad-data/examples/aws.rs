use clap::Parser;

#[cfg(not(feature = "aws"))]
fn main() {
    println!("This example requires the \"aws\" feature to be enabled.");
}

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

#[cfg(feature = "aws")]
#[tokio::main]
async fn main() -> nexrad_data::result::Result<()> {
    use chrono::{NaiveDate, NaiveTime};
    use nexrad_data::aws::archive::{download_file, list_files};
    use std::fs::{create_dir, File};
    use std::io::Write;
    use std::path::Path;

    let cli = Cli::parse();

    let site = &cli.site;
    let date = NaiveDate::parse_from_str(&cli.date, "%Y-%m-%d").expect("is valid date");
    let start_time =
        NaiveTime::parse_from_str(&cli.start_time, "%H:%M").expect("start is valid time");
    let stop_time = NaiveTime::parse_from_str(&cli.stop_time, "%H:%M").expect("stop is valid time");

    println!("Listing files for {} on {}...", site, date);
    let file_ids = list_files(site, &date).await?;

    if file_ids.is_empty() {
        println!("No files found for the specified date/site to download.");
        return Ok(());
    }

    println!("Found {} files.", file_ids.len());

    let start_index = get_nearest_file_index(&file_ids, start_time);
    println!(
        "Nearest file to start of {:?} is {:?}.",
        start_time,
        file_ids[start_index].name()
    );

    let stop_index = get_nearest_file_index(&file_ids, stop_time);
    println!(
        "Nearest file to stop of {:?} is {:?}.",
        stop_time,
        file_ids[stop_index].name()
    );

    println!("Downloading {} files...", stop_index - start_index + 1);

    for file_id in file_ids
        .iter()
        .skip(start_index)
        .take(stop_index - start_index + 1)
    {
        println!("Downloading file \"{}\"...", file_id.name());
        let file = download_file(file_id.clone()).await?;

        println!("Data file size (bytes): {}", file.data().len());

        if !Path::new("downloads").exists() {
            println!("Creating downloads directory...");
            create_dir("downloads").expect("create downloads directory");
        }

        println!("Writing file to disk as: {}", file_id.name());
        let mut downloaded_file =
            File::create(format!("downloads/{}", file_id.name())).expect("create file");
        downloaded_file
            .write_all(file.data().as_slice())
            .expect("write file");
    }

    println!("Downloaded {} files.", stop_index - start_index + 1);

    Ok(())
}

/// Returns the index of the file with the nearest time to the provided start time.
#[cfg(feature = "aws")]
fn get_nearest_file_index(
    files: &Vec<nexrad_data::aws::archive::Identifier>,
    start_time: chrono::NaiveTime,
) -> usize {
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
