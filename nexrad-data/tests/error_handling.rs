use nexrad_data::result;

#[test]
fn test_error_display_formatting() {
    use std::error::Error;

    let io_error = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "test error");
    let file_error = result::Error::FileError(io_error);
    let display_output = format!("{}", file_error);
    assert!(display_output.contains("data file IO error"));
    assert!(file_error.source().is_some());
}

#[cfg(all(feature = "serde", feature = "bincode"))]
#[test]
fn test_deserialization_error() {
    use std::io::Cursor;

    let invalid_data = vec![0xFF; 10]; // Invalid data for header
    let mut cursor = Cursor::new(invalid_data);

    let result = nexrad_data::volume::Header::deserialize(&mut cursor);
    assert!(result.is_err(), "Should fail to deserialize invalid data");

    let error = result.unwrap_err();
    match error {
        result::Error::DeserializationError(_) => {
            let display = format!("{}", error);
            assert!(display.contains("file deserialization error"));
        }
        _ => panic!("Expected DeserializationError, got: {:?}", error),
    }
}

#[cfg(feature = "bzip2")]
#[test]
fn test_uncompressed_data_error() {
    let uncompressed_data = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let record = nexrad_data::volume::Record::new(uncompressed_data);
    assert!(!record.compressed());

    let decompress_result = record.decompress();
    assert!(
        decompress_result.is_err(),
        "Should fail to decompress uncompressed data"
    );

    let error = decompress_result.unwrap_err();
    match error {
        result::Error::UncompressedDataError => {
            let display = format!("{}", error);
            assert!(display.contains("error decompressing uncompressed data"));
        }
        _ => panic!("Expected UncompressedDataError, got: {:?}", error),
    }
}

#[cfg(feature = "nexrad-decode")]
#[test]
fn test_compressed_data_decode_error() {
    const TEST_NEXRAD_FILE: &[u8] = include_bytes!("../../downloads/KDMX20220305_232324_V06");

    let volume = nexrad_data::volume::File::new(TEST_NEXRAD_FILE.to_vec());
    let records = volume.records();
    let compressed_record = &records[0];
    assert!(
        compressed_record.compressed(),
        "Test record should be compressed"
    );

    let messages_result = compressed_record.messages();
    assert!(
        messages_result.is_err(),
        "Should fail to decode compressed data"
    );

    let error = messages_result.unwrap_err();
    match error {
        result::Error::CompressedDataError => {
            let display = format!("{}", error);
            assert!(display.contains("compressed data cannot be decoded"));
        }
        _ => panic!("Expected CompressedDataError, got: {:?}", error),
    }
}
