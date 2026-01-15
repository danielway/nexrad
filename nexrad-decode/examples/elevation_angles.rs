use clap::Parser;
use log::{debug, info, LevelFilter};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the Archive II file to process
    #[arg(required = true)]
    file_path: String,

    /// Path to save the output CSV file (defaults to 'elevation_angles.csv')
    #[arg(default_value = "elevation_angles.csv")]
    output_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_module("reqwest::connect", LevelFilter::Info)
        .init();

    // Parse command line arguments
    let cli = Cli::parse();
    let file_path = &cli.file_path;
    let output_path = &cli.output_path;

    info!("Processing file: {file_path}");

    // Read the file
    let mut file =
        File::open(file_path).unwrap_or_else(|_| panic!("Failed to open file: {file_path}"));
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Parse the Archive II file
    let volume_file = nexrad_data::volume::File::new(buffer);

    // Create a HashMap to store elevation angles by elevation and azimuth numbers
    // Key: (elevation_number, azimuth_number), Value: elevation_angle
    let mut elevation_angles: HashMap<(u8, u16), f32> = HashMap::new();

    // Maximum elevation number and azimuth number for CSV dimensions
    let mut max_elevation_num = 0;
    let mut max_azimuth_num = 0;

    // Process all records in the file
    for mut record in volume_file.records() {
        debug!("Processing record...");
        if record.compressed() {
            debug!("Decompressing LDM record...");
            record = record.decompress().expect("Failed to decompress record");
        }

        // Extract messages from the record
        let messages = record.messages()?;

        // Process each message to extract elevation angles
        for message in messages {
            // Check if the message is a Digital Radar Data message
            if let nexrad_decode::messages::MessageContents::DigitalRadarData(digital_data) =
                message.contents()
            {
                // Access header information where the elevation data is stored
                let header = digital_data.header();

                let elevation_num = header.elevation_number();
                let azimuth_num = header.azimuth_number();
                let elevation_angle = header.elevation_angle_raw();

                // Update max values for dimensions
                if elevation_num > max_elevation_num {
                    max_elevation_num = elevation_num;
                }
                if azimuth_num > max_azimuth_num {
                    max_azimuth_num = azimuth_num;
                }

                // Store elevation angle information
                elevation_angles.insert((elevation_num, azimuth_num), elevation_angle);
                debug!(
                    "Elevation: {elevation_num}, Azimuth Number: {azimuth_num}, Elevation Angle: {elevation_angle}"
                );
            }
        }
    }

    info!(
        "Found elevation angles for {} elevation-azimuth combinations",
        elevation_angles.len()
    );
    info!("Maximum elevation number: {max_elevation_num}");
    info!("Maximum azimuth number: {max_azimuth_num}");

    // Create and write the CSV file
    let mut output_file = File::create(output_path)?;

    // Write header with elevation numbers
    write!(output_file, "azimuth_num")?;
    for elev in 1..=max_elevation_num {
        write!(output_file, ",elev_{elev}")?;
    }
    writeln!(output_file)?;

    // Write rows for each azimuth number
    for azimuth_num in 1..=max_azimuth_num {
        write!(output_file, "{azimuth_num}")?;

        // For each elevation in this azimuth number, write the elevation angle
        for elev_num in 1..=max_elevation_num {
            match elevation_angles.get(&(elev_num, azimuth_num)) {
                Some(angle) => write!(output_file, ",{angle:.2}")?,
                None => write!(output_file, ",")?, // Empty value if no data for this combination
            }
        }
        writeln!(output_file)?;
    }

    info!("CSV file created successfully: {output_path}");

    Ok(())
}
