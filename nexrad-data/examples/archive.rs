use clap::Parser;
use log::{debug, info, trace, LevelFilter};

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

#[cfg(all(feature = "aws", feature = "decode"))]
#[tokio::main]
async fn main() -> nexrad_data::result::Result<()> {
    use chrono::Utc;
    use chrono::{NaiveDate, NaiveTime};
    use nexrad_data::aws::archive::{download_file, list_files};
    use nexrad_data::volume::File;
    use std::fs::create_dir;
    use std::io::Read;
    use std::io::Write;
    use std::path::Path;

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .filter_module("reqwest::connect", LevelFilter::Info)
        .init();

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
        let file = if Path::new(&format!("downloads/{}", file_id.name())).exists() {
            println!("File \"{}\" already downloaded.", file_id.name());
            let mut file =
                std::fs::File::open(format!("downloads/{}", file_id.name())).expect("open file");

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).expect("read file");

            File::new(buffer)
        } else {
            println!("Downloading file \"{}\"...", file_id.name());
            let file = download_file(file_id.clone()).await?;

            println!("Data file size (bytes): {}", file.data().len());
            if !Path::new("downloads").exists() {
                println!("Creating downloads directory...");
                create_dir("downloads").expect("create downloads directory");
            }

            if !Path::new("downloads").exists() {
                println!("Creating downloads directory...");
                create_dir("downloads").expect("create downloads directory");
            }

            println!("Writing file to disk as: {}", file_id.name());
            let mut downloaded_file =
                std::fs::File::create(format!("downloads/{}", file_id.name()))
                    .expect("create file");

            downloaded_file
                .write_all(file.data().as_slice())
                .expect("write file");

            file
        };

        println!("Data file size (bytes): {}", file.data().len());

        let records = file.records();
        debug!(
            "Volume start chunk with {} records. Header: {:?}",
            records.len(),
            file.header()
        );

        records
            .into_iter()
            .for_each(|record| decode_record(record, Utc::now()));
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

#[cfg(all(feature = "aws", feature = "decode"))]
fn decode_record(
    mut record: nexrad_data::volume::Record,
    download_time: chrono::DateTime<chrono::Utc>,
) {
    use nexrad_decode::messages::MessageType;
    use nexrad_decode::messages::{decode_messages, Message};
    use std::collections::HashMap;
    use std::io::Cursor;

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
    let messages = decode_messages(&mut reader).expect("Failed to decode messages");
    for message in messages {
        let message_header = message.header;

        if first_message_time.is_none() {
            first_message_time = message_header.date_time();
        }

        let message_type = message_header.message_type();
        let count = message_type_counts.get(&message_type).unwrap_or(&0) + 1;
        message_type_counts.insert(message_type, count);

        match message.message {
            Message::DigitalRadarData(m31) => {
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
                            let count =
                                current_scan_data.data_types.get(data_type).unwrap_or(&0) + 1;
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
            }
            _ => {
                if let Some(scan_data) = scan_data.take() {
                    all_scans.push(format!("{}", scan_data));
                }
            }
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
    data_types: std::collections::HashMap<String, usize>,
}

impl std::fmt::Display for ScanData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ScanData {{ azimuth: {:.0}-{:.0}, elevation: {}, data_types: {:?} }}",
            self.start_azimuth, self.end_azimuth, self.elevation, self.data_types
        )
    }
}
