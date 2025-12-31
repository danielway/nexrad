use insta::assert_debug_snapshot;
use nexrad_data::volume;
use nexrad_decode::messages::decode_messages;

const TEST_NEXRAD_FILE: &[u8] = include_bytes!("../../downloads/KDMX20220305_232324_V06");

#[test]
fn test_decode_volume() {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());
    assert_debug_snapshot!("Volume Header", volume.header());

    for (record_number, mut record) in volume.records().into_iter().enumerate() {
        if record.compressed() {
            record = record.decompress().expect("decompresses records");
        }

        let messages = decode_messages(record.data()).expect("decodes successfully");

        assert_debug_snapshot!(format!("Record {}", record_number), messages);
    }
}
