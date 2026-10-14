use insta::assert_debug_snapshot;

use crate::messages::decode_messages;
#[cfg(feature = "model")]
use crate::messages::MessageContents;

const TEST_DATA: &[u8] = include_bytes!("../../../tests/data/messages/digital_radar_data_full.bin");

/// Tests decoding of a single Digital Radar Data message (type 31).
#[test]
fn test_decode_digital_radar_data() {
    let messages = decode_messages(TEST_DATA).expect("decodes successfully");

    assert_eq!(messages.len(), 1, "expected exactly one message");
    assert_debug_snapshot!(messages[0]);
}

/// Tests that converting a Message Type 31 to a model Radial carries the radial data block's
/// Nyquist velocity and unambiguous range.
#[cfg(feature = "model")]
#[test]
fn test_radial_carries_nyquist_velocity_and_unambiguous_range() {
    let messages = decode_messages(TEST_DATA).expect("decodes successfully");
    let drd = match messages[0].contents() {
        MessageContents::DigitalRadarData(drd) => drd,
        other => panic!("expected DigitalRadarData, got {:?}", other),
    };
    let block = drd
        .radial_data_block()
        .expect("fixture has a radial data block")
        .inner();

    let radial = drd.radial().expect("converts to radial");
    assert_eq!(
        radial.nyquist_velocity_mps(),
        Some(block.nyquist_velocity_mps())
    );
    assert_eq!(
        radial.unambiguous_range_km(),
        Some(block.unambiguous_range_km())
    );

    let owned = drd
        .as_ref()
        .clone()
        .into_radial()
        .expect("converts to radial");
    assert_eq!(owned.nyquist_velocity_mps(), radial.nyquist_velocity_mps());
    assert_eq!(owned.unambiguous_range_km(), radial.unambiguous_range_km());
}
