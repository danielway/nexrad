use nexrad_data::archive;
use nexrad_data::archive::Record;
use nexrad_decode::messages::digital_radar_data::decode_digital_radar_data;
use nexrad_decode::messages::message_header::MessageHeader;
use nexrad_decode::messages::{decode_message_header, MessageType};
use std::collections::HashMap;
use std::fs::{read, read_dir};
use std::io::{Cursor, Seek, SeekFrom};

#[cfg(not(feature = "aws"))]
fn main() {
    println!("This example requires the \"aws\" feature to be enabled.");
}

#[cfg(feature = "aws")]
fn main() {
    let files = read_dir("downloads").unwrap();
    for file in files {
        let path = file.unwrap().path();
        let file = path.to_str().unwrap();
        if file.ends_with("S") {
            decode_start_chunk(file);
        } else {
            decode_non_start_chunk(file);
        }
    }
}

fn decode_start_chunk(file: &str) {
    println!("Decoding start chunk: {}", file);
    let start = read(file).unwrap();
    let file = archive::File::new(start);

    println!("  Archive header: {:?}", file.header().unwrap());

    let records = file.records();
    println!("  Found {} records in start of volume file.", records.len());

    for record in records {
        decode_record(record);
    }

    println!("  Decoded all messages in start of volume file.\n");
}

fn decode_non_start_chunk(file: &str) {
    println!("Decoding chunk: {}", file);
    let data = read(file).unwrap();

    // Non-start chunks are just records without a volume header
    let record = Record::new(data);
    decode_record(record);

    println!("  Decoded all messages in chunk file.\n");
}

fn decode_record(mut record: Record) {
    println!("  Decoding record...");
    if record.compressed() {
        println!("    Decompressing record...");
        let decompressed_record = record.decompress().unwrap();
        record = decompressed_record;
    }

    let mut message_type_counts = HashMap::new();

    let mut reader = Cursor::new(record.data());
    while reader.position() < reader.get_ref().len() as u64 {
        let message_header = decode_message_header(&mut reader).unwrap();

        let message_type = message_header.message_type();
        let count = message_type_counts.get(&message_type).unwrap_or(&0) + 1;
        message_type_counts.insert(message_type, count);

        if message_header.message_type() == MessageType::RDADigitalRadarData {
            // todo: message type counts are off and we're having a read error on an M31 for
            //       downloads/20240813-123330-005-I

            // Decoding the message will advance the reader
            decode_digital_radar_data(&mut reader).unwrap();
        } else {
            // Non-M31 messages are 2432 bytes long, including the header
            reader
                .seek(SeekFrom::Current(2432 - size_of::<MessageHeader>() as i64))
                .unwrap();
        }
    }

    for (message_type, count) in message_type_counts {
        println!("    {:?}: {}", message_type, count);
    }
}
