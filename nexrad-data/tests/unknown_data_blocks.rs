//! A volume whose radar data messages carry a data block this version does not recognise still
//! builds a scan.

#![cfg(feature = "model")]

use bzip2::write::BzEncoder;
use bzip2::Compression;
use nexrad_data::volume::File;
use std::io::Write;

const VOLUME_COVERAGE_PATTERN: &[u8] =
    include_bytes!("../../nexrad-decode/tests/data/messages/volume_coverage_pattern.bin");
const DIGITAL_RADAR_DATA: &[u8] =
    include_bytes!("../../nexrad-decode/tests/data/messages/digital_radar_data_full.bin");

/// Message header size, and offsets of fields within a type 31 message.
const DRD_HEADER: usize = 28;
const SEGMENT_SIZE: usize = 12;
const DRD_RADIAL_LENGTH: usize = DRD_HEADER + 18;
const DRD_BLOCK_COUNT: usize = DRD_HEADER + 30;
const DRD_POINTERS: usize = DRD_HEADER + 32;

fn read_u16(data: &[u8], at: usize) -> usize {
    u16::from_be_bytes([data[at], data[at + 1]]) as usize
}

fn write_u16(data: &mut [u8], at: usize, value: usize) {
    data[at..at + 2].copy_from_slice(&(value as u16).to_be_bytes());
}

/// The type 31 fixture with a copy of its last (CFP) block appended under the name `XYZ`.
fn radar_data_with_unknown_block() -> Vec<u8> {
    let count = read_u16(DIGITAL_RADAR_DATA, DRD_BLOCK_COUNT);
    let pointers: Vec<usize> = (0..count)
        .map(|i| {
            let at = DRD_POINTERS + 4 * i;
            u32::from_be_bytes(DIGITAL_RADAR_DATA[at..at + 4].try_into().unwrap()) as usize
        })
        .collect();
    let radial_length = read_u16(DIGITAL_RADAR_DATA, DRD_RADIAL_LENGTH);
    let message_end = DRD_HEADER + radial_length;

    let mut block = DIGITAL_RADAR_DATA[DRD_HEADER + pointers[count - 1]..message_end].to_vec();
    assert_eq!(&block[..4], b"DCFP");
    block[1..4].copy_from_slice(b"XYZ");

    let mut data = DIGITAL_RADAR_DATA[..DRD_POINTERS].to_vec();
    for pointer in pointers.iter().chain([&radial_length]) {
        data.extend_from_slice(&(*pointer as u32 + 4).to_be_bytes());
    }
    data.extend_from_slice(&DIGITAL_RADAR_DATA[DRD_POINTERS + 4 * count..message_end]);
    data.extend_from_slice(&block);

    let added = 4 + block.len();
    write_u16(&mut data, DRD_BLOCK_COUNT, count + 1);
    write_u16(&mut data, DRD_RADIAL_LENGTH, radial_length + added);
    let segment_size = read_u16(DIGITAL_RADAR_DATA, SEGMENT_SIZE);
    write_u16(&mut data, SEGMENT_SIZE, segment_size + added / 2);
    data
}

/// A volume file holding one bzip2-compressed LDM record with the given messages.
fn volume_file(messages: &[&[u8]]) -> File {
    let mut encoder = BzEncoder::new(Vec::new(), Compression::default());
    for message in messages {
        encoder.write_all(message).unwrap();
    }
    let compressed = encoder.finish().unwrap();

    let mut data = Vec::new();
    data.extend_from_slice(b"AR2V0006.001");
    data.extend_from_slice(&19_000u32.to_be_bytes());
    data.extend_from_slice(&0u32.to_be_bytes());
    data.extend_from_slice(b"KDMX");
    data.extend_from_slice(&(compressed.len() as i32).to_be_bytes());
    data.extend_from_slice(&compressed);
    File::new(data)
}

#[test]
fn scan_builds_from_a_volume_with_an_unknown_data_block() {
    let modified = radar_data_with_unknown_block();
    let scan = volume_file(&[VOLUME_COVERAGE_PATTERN, &modified])
        .scan()
        .expect("an unknown data moment block does not fail the scan");
    let expected = volume_file(&[VOLUME_COVERAGE_PATTERN, DIGITAL_RADAR_DATA])
        .scan()
        .expect("the unmodified volume builds a scan");

    assert_eq!(scan.sweeps().len(), 1);
    assert_eq!(scan, expected);
}
