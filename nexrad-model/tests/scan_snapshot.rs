use insta::assert_debug_snapshot;
use nexrad_data::volume;

const TEST_NEXRAD_FILE: &[u8] = include_bytes!("../../downloads/KDMX20220305_232324_V06");

#[test]
fn test_decode_map_scan() {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());
    let scan = volume.scan().expect("decodes into scan model");
    assert_debug_snapshot!(scan);
}
