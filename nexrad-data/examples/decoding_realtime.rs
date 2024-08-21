#[cfg(not(feature = "aws"))]
fn main() {
    println!("This example requires the \"aws\" feature to be enabled.");
}

#[cfg(feature = "aws")]
fn main() {
    use nexrad_data::aws::realtime::Chunk;
    use std::fs::{read, read_dir};

    let files = read_dir("downloads").unwrap();
    let file_names = files
        .map(|file| file.unwrap().path().to_str().unwrap().to_string())
        .collect::<Vec<_>>();

    for file in file_names {
        println!("Reading chunk file: {}", file);
        match Chunk::new(read(&file).unwrap()) {
            Ok(chunk) => match chunk {
                Chunk::Start(file) => {
                    println!("  Start chunk, volume header: {:?}", file.header());

                    let records = file.records();
                    let first_record = records.into_iter().next().unwrap();
                    decode_record(first_record);
                }
                Chunk::IntermediateOrEnd(record) => {
                    println!("  Intermediate or end chunk:");
                    decode_record(record);
                }
            },
            Err(e) => {
                println!("Error reading chunk file: {:?}", e);
            }
        }
    }
}

#[cfg(feature = "aws")]
fn decode_record(mut record: nexrad_data::volume::Record) {
    use nexrad_decode::messages::digital_radar_data::decode_digital_radar_data;
    use nexrad_decode::messages::message_header::MessageHeader;
    use nexrad_decode::messages::{decode_message_header, MessageType};
    use std::collections::HashMap;
    use std::io::{Cursor, Seek, SeekFrom};

    println!("  Decoding record...");
    if record.compressed() {
        println!("    Decompressing record...");
        let decompressed_record = record.decompress().unwrap();
        record = decompressed_record;
    }

    let mut message_type_counts = HashMap::new();

    // println!("    Headers:");
    let mut reader = Cursor::new(record.data());
    while reader.position() < reader.get_ref().len() as u64 {
        let message_header = decode_message_header(&mut reader).unwrap();
        // println!("      {:?}", message_header);

        let message_type = message_header.message_type();
        let count = message_type_counts.get(&message_type).unwrap_or(&0) + 1;
        message_type_counts.insert(message_type, count);

        if message_header.message_type() == MessageType::RDADigitalRadarDataGenericFormat {
            // Decoding the message will advance the reader
            decode_digital_radar_data(&mut reader).unwrap();
        } else {
            // Non-M31 messages are 2432 bytes long, including the header
            reader
                .seek(SeekFrom::Current(2432 - size_of::<MessageHeader>() as i64))
                .unwrap();
        }
    }

    println!("    Message type counts:");
    for (message_type, count) in message_type_counts {
        println!("      {:?}: {}", message_type, count);
    }
}
