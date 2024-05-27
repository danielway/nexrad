//!
//! examples/inspect
//!
//! This example loads, decodes, and inspects a data file printing various metadata and statistics.
//!
//! Usage: cargo run --example inspect -- <file or directory>
//!

use nexrad::decompress::decompress_and_decode_archive2_file;
use nexrad::model::messages::MessageType::RDADigitalRadarDataGenericFormat;
use nexrad::model::messages::{Message, MessageType};
use nexrad::model::Archive2File;
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut path = "downloads/KDMX20220305_233003_V06";
    if args.len() >= 2 {
        path = &args[1];
    }

    let path_metadata = fs::metadata(path).expect("file exists");
    let file_names = if path_metadata.is_dir() {
        fs::read_dir(path)
            .expect("directory exists")
            .map(|entry| {
                let file_name = entry.expect("entry exists").file_name();
                format!(
                    "{}/{}",
                    path,
                    file_name.to_str().expect("file name is valid")
                )
            })
            // These don't seem to be Archive2 files, so skip them
            .filter(|file_name| {
                let is_mdm = file_name.ends_with("MDM");
                if is_mdm {
                    println!("Skipping non-Archive2 file: {}", file_name);
                }
                !is_mdm
            })
            .collect()
    } else {
        vec![path.to_string()]
    };

    println!("Loading and decoding {} file(s)...", file_names.len());
    let mut decoded_files = file_names
        .iter()
        .map(|file_name| {
            let file = fs::read(file_name).expect("file exists");

            println!(
                "Loaded file \"{}\" of size {} bytes.",
                file_name,
                file.len()
            );

            let mut cursor = Cursor::new(file.as_slice());
            let size = file.len() as u64;

            let decoded_file =
                decompress_and_decode_archive2_file(&mut cursor, size).expect("decodes file");

            println!(
                "Decoded file with {} messages.",
                decoded_file.messages.len()
            );

            decoded_file
        })
        .collect::<Vec<Archive2File>>();

    decoded_files.sort_by(|a, b| a.header.date_time().cmp(&b.header.date_time()));

    println!("Loaded and decoded {} files.", decoded_files.len());

    for decoded_file in &decoded_files {
        println!("Inspecting file: {}", decoded_file.header.tape_filename());
        println!("  Archive 2 header: {:?}", decoded_file.header);

        let counts_by_message_type =
            decoded_file
                .messages
                .iter()
                .fold(HashMap::new(), |mut acc, message| {
                    let message_type = message.header.message_type();
                    if acc.contains_key(&message_type) {
                        acc.insert(message_type, acc.get(&message_type).unwrap() + 1);
                    } else {
                        acc.insert(message_type, 1);
                    }
                    acc
                });

        let mut ordered_counts_by_message_type: Vec<(MessageType, i32)> =
            counts_by_message_type.into_iter().collect();
        ordered_counts_by_message_type
            .sort_by(|(message_type_a, _), (message_type_b, _)| message_type_a.cmp(message_type_b));

        println!("  Message counts by type:");
        for (message_type, count) in ordered_counts_by_message_type {
            println!("    {:?}: {}", message_type, count);

            if message_type == RDADigitalRadarDataGenericFormat {
                let mut radar_data_messages = decoded_file
                    .messages
                    .iter()
                    .filter(|m| m.header.message_type() == RDADigitalRadarDataGenericFormat);

                let example_message = radar_data_messages.next().expect("has message 31");

                let mut elevations = HashSet::new();
                let mut products = HashSet::new();

                if let Message::DigitalRadarData(message) = &example_message.message {
                    elevations.insert(message.header.elevation_number);

                    if message.reflectivity_data_block.is_some() {
                        products.insert("REF");
                    }

                    if message.velocity_data_block.is_some() {
                        products.insert("VEL");
                    }

                    if message.spectrum_width_data_block.is_some() {
                        products.insert("SW");
                    }

                    if message.differential_reflectivity_data_block.is_some() {
                        products.insert("ZDR");
                    }

                    if message.differential_phase_data_block.is_some() {
                        products.insert("PHI");
                    }

                    if message.correlation_coefficient_data_block.is_some() {
                        products.insert("RHO");
                    }

                    if message.specific_diff_phase_data_block.is_some() {
                        products.insert("CFP");
                    }
                }

                println!(
                    "      Sample: elevations={:?}, products={:?}",
                    elevations, products
                );
            }
        }
    }
}
