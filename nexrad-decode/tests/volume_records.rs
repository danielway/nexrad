use nexrad_data::volume;

const TEST_NEXRAD_FILE: &[u8] = include_bytes!("KDMX20220305_232324_V06");

#[test]
fn splits_volume_records() {
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
fn decodes_volume_header() {
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
