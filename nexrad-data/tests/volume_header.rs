#![cfg(all(feature = "serde", feature = "bincode"))]

use nexrad_data::volume;
use std::io::Cursor;

const TEST_NEXRAD_FILE: &[u8] = include_bytes!("KDMX20220305_232324_V06");

#[test]
fn test_header_deserialization_success() {
    let mut cursor = Cursor::new(TEST_NEXRAD_FILE);
    let header =
        volume::Header::deserialize(&mut cursor).expect("Header deserialization should succeed");

    let expected_volume_datetime = chrono::DateTime::parse_from_rfc3339("2022-03-05T23:23:24.299Z")
        .unwrap()
        .with_timezone(&chrono::Utc);

    assert_eq!(header.tape_filename(), Some("AR2V0006.".to_string()));
    assert_eq!(header.extension_number(), Some("879".to_string()));
    assert_eq!(header.date_time(), Some(expected_volume_datetime));
    assert_eq!(header.icao_of_radar(), Some("KDMX".to_string()));
}

#[test]
fn test_header_deserialization_insufficient_data() {
    let short_data = vec![0u8; 10];
    let mut cursor = Cursor::new(short_data);

    let result = volume::Header::deserialize(&mut cursor);
    assert!(result.is_err(), "Should fail with insufficient data");
}
