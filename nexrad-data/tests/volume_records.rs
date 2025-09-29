use nexrad_data::volume;

const TEST_NEXRAD_FILE: &[u8] = include_bytes!("KDMX20220305_232324_V06");

#[test]
fn test_volume_record_splitting() {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());
    let records = volume.records().to_vec();

    let expected_sizes = vec![
        2335, 233467, 165644, 208109, 261787, 201050, 178271, 88007, 54902, 91951, 112300, 68236,
        83956, 156998, 123992, 193491, 210583, 137846, 192423, 41764, 57291, 101640, 65652, 59666,
        87053, 87089, 131084, 225255, 112429, 154255, 163862, 41601, 82464, 79094, 44544, 78986,
        48425, 176495, 142862, 128631, 198090, 144577, 116768, 209163, 160277, 95847, 197053,
        152302, 97937, 174047, 121761, 113719, 175451, 221153, 184761, 166211, 250220, 241865,
        74673, 95312, 55774, 73970, 111042, 80020, 175182, 183634, 120029, 159418, 232987, 145785,
        80566, 60989, 43272, 81658, 94158, 54786, 185412, 101308, 122282, 197176, 160829, 121687,
        71336, 30670, 57325, 106921, 49579, 65386, 80238, 147372, 128985, 77999, 137932, 106809,
        94068, 111304, 80761, 87093, 82824, 72393, 82626, 65251, 63761, 68136, 61350, 56750,
    ];

    assert_eq!(records.len(), expected_sizes.len());

    for (i, record) in records.iter().enumerate() {
        assert!(record.compressed(), "Record {} should be compressed", i);
        assert_eq!(
            record.data().len(),
            expected_sizes[i],
            "Record {} size mismatch",
            i
        );
    }
}

#[test]
fn test_volume_header_decoding() {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());
    let header = volume.header().expect("Failed to parse header");

    let expected_volume_datetime = chrono::DateTime::parse_from_rfc3339("2022-03-05T23:23:24.299Z")
        .unwrap()
        .with_timezone(&chrono::Utc);

    assert_eq!(header.tape_filename(), Some("AR2V0006.".to_string()));
    assert_eq!(header.extension_number(), Some("879".to_string()));
    assert_eq!(header.date_time(), Some(expected_volume_datetime));
    assert_eq!(header.icao_of_radar(), Some("KDMX".to_string()));
}

#[test]
fn test_record_construction_and_data_access() {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());
    let records = volume.records();

    let first_record = &records[0];
    let first_record_data = first_record.data().to_vec();

    let owned_record = volume::Record::new(first_record_data.clone());
    assert_eq!(owned_record.data(), first_record_data.as_slice());
    assert_eq!(owned_record.data().len(), 2335);

    let borrowed_record = volume::Record::from_slice(&first_record_data);
    assert_eq!(borrowed_record.data(), first_record_data.as_slice());
    assert_eq!(borrowed_record.data().len(), 2335);

    assert_eq!(owned_record.data(), borrowed_record.data());
}

#[test]
fn test_record_compression_detection() {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());
    let records = volume.records();

    let first_record = &records[0];
    assert_eq!(first_record.compressed(), true);

    let compressed_count = records.iter().filter(|r| r.compressed()).count();
    let total_count = records.len();
    assert_eq!(compressed_count, 106);
    assert_eq!(total_count, 106);

    let test_data_uncompressed = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let uncompressed_record = volume::Record::new(test_data_uncompressed);
    assert!(!uncompressed_record.compressed());

    let test_data_compressed = vec![0, 1, 2, 3, b'B', b'Z', 6, 7, 8, 9];
    let compressed_record = volume::Record::new(test_data_compressed);
    assert!(compressed_record.compressed());

    let short_data = vec![0, 1, 2, 3, 4];
    let short_record = volume::Record::new(short_data);
    assert!(!short_record.compressed());
}

#[test]
fn test_record_decompression() {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());
    let records = volume.records();

    let first_record = &records[0];
    assert!(
        first_record.compressed(),
        "First record should be compressed"
    );

    let decompressed = first_record
        .decompress()
        .expect("Decompression should succeed");
    assert!(decompressed.data().len() > first_record.data().len());
    assert!(
        !decompressed.compressed(),
        "Decompressed record should not be compressed"
    );
    assert_eq!(decompressed.data().len(), 325888);

    let uncompressed_data = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let uncompressed_record = volume::Record::new(uncompressed_data);
    assert!(!uncompressed_record.compressed());

    let decompress_result = uncompressed_record.decompress();
    assert!(
        decompress_result.is_err(),
        "Should fail to decompress uncompressed data"
    );
}

#[test]
fn test_record_message_decoding() {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());
    let records = volume.records();

    let first_record = &records[0];
    let decompressed = first_record
        .decompress()
        .expect("Decompression should succeed");

    let messages = decompressed
        .messages()
        .expect("Message decoding should succeed");
    assert_eq!(messages.len(), 134);

    let first_message = &messages[0];
    assert_eq!(first_message.header().message_size_bytes(), 2416);
    assert_eq!(
        first_message.header().message_type(),
        nexrad_decode::messages::MessageType::RDAClutterFilterMap
    );

    let compressed_messages_result = first_record.messages();
    assert!(
        compressed_messages_result.is_err(),
        "Should fail to decode messages from compressed data"
    );
}

#[test]
fn test_full_volume_record_decoding() {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());
    let records = volume.records();

    let mut total_messages = 0;
    let mut digital_radar_messages = 0;
    let mut clutter_filter_messages = 0;
    let mut status_messages = 0;

    for (i, record) in records.iter().enumerate() {
        assert!(record.compressed(), "Record {} should be compressed", i);

        let decompressed = record.decompress().expect("Decompression should succeed");
        assert!(
            !decompressed.compressed(),
            "Decompressed record {} should not be compressed",
            i
        );
        assert!(
            decompressed.data().len() > record.data().len(),
            "Decompressed record {} should be larger",
            i
        );

        let messages = decompressed
            .messages()
            .expect("Message decoding should succeed");
        total_messages += messages.len();

        for message in &messages {
            match message.header().message_type() {
                nexrad_decode::messages::MessageType::RDADigitalRadarDataGenericFormat => {
                    digital_radar_messages += 1
                }
                nexrad_decode::messages::MessageType::RDAClutterFilterMap => {
                    clutter_filter_messages += 1
                }
                nexrad_decode::messages::MessageType::RDAStatusData => status_messages += 1,
                _ => {}
            }
        }
    }

    assert_eq!(total_messages, 12736);
    assert_eq!(digital_radar_messages, 12600);
    assert_eq!(clutter_filter_messages, 5);
    assert_eq!(status_messages, 3);
}
