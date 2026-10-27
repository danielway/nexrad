//! Digital radar data messages with data blocks this version does not recognise.
//!
//! A later ICD build may add a data moment (CFP was added this way). These tests synthesise a
//! type 31 message with an extra block appended and check that it still decodes.

use nexrad_decode::messages::digital_radar_data::DataBlockType;
use nexrad_decode::messages::MessageHeader;
use nexrad_decode::{decode_messages, Error, MessageContents};

const DIGITAL_RADAR_DATA: &[u8] = include_bytes!("data/messages/digital_radar_data_full.bin");

/// Offset of the digital radar data header within a type 31 message.
const DRD_HEADER: usize = MessageHeader::SIZE;
/// Offset of the data block count within a type 31 message.
const DRD_BLOCK_COUNT: usize = DRD_HEADER + 30;
/// Offset of the first data block pointer within a type 31 message.
const DRD_POINTERS: usize = DRD_HEADER + 32;
/// Offset of the radial length within a type 31 message.
const DRD_RADIAL_LENGTH: usize = DRD_HEADER + 18;
/// Offset of the segment size (in halfwords) within the message header.
const SEGMENT_SIZE: usize = 12;

fn read_u16(data: &[u8], at: usize) -> u16 {
    u16::from_be_bytes([data[at], data[at + 1]])
}

fn write_u16(data: &mut [u8], at: usize, value: u16) {
    data[at..at + 2].copy_from_slice(&value.to_be_bytes());
}

fn pointers(data: &[u8]) -> Vec<usize> {
    let count = read_u16(data, DRD_BLOCK_COUNT) as usize;
    (0..count)
        .map(|i| {
            let at = DRD_POINTERS + 4 * i;
            u32::from_be_bytes(data[at..at + 4].try_into().expect("four bytes")) as usize
        })
        .collect()
}

/// Returns the fixture with `block` appended as a new data block: the pointer table grows by one
/// entry, every existing block moves back four bytes, and the radial length and segment size grow
/// to cover the new pointer and block.
fn with_extra_block(block: &[u8]) -> Vec<u8> {
    assert_eq!(
        block.len() % 2,
        0,
        "messages are a whole number of halfwords"
    );
    let old = pointers(DIGITAL_RADAR_DATA);
    let count = old.len();
    let table_end = DRD_POINTERS + 4 * count;
    let radial_length = read_u16(DIGITAL_RADAR_DATA, DRD_RADIAL_LENGTH) as usize;

    let mut data = DIGITAL_RADAR_DATA[..DRD_POINTERS].to_vec();
    for pointer in &old {
        data.extend_from_slice(&(*pointer as u32 + 4).to_be_bytes());
    }
    data.extend_from_slice(&(radial_length as u32 + 4).to_be_bytes());
    data.extend_from_slice(&DIGITAL_RADAR_DATA[table_end..DRD_HEADER + radial_length]);
    data.extend_from_slice(block);

    write_u16(&mut data, DRD_BLOCK_COUNT, count as u16 + 1);
    write_u16(
        &mut data,
        DRD_RADIAL_LENGTH,
        (radial_length + 4 + block.len()) as u16,
    );
    let segment_size = read_u16(DIGITAL_RADAR_DATA, SEGMENT_SIZE) as usize;
    write_u16(
        &mut data,
        SEGMENT_SIZE,
        (segment_size + (4 + block.len()) / 2) as u16,
    );
    data
}

/// The fixture's CFP block, renamed and given the type byte `block_type`.
fn renamed_moment_block(block_type: u8, name: &[u8; 3]) -> Vec<u8> {
    let pointers = pointers(DIGITAL_RADAR_DATA);
    let start = DRD_HEADER + *pointers.last().expect("a block");
    let end = DRD_HEADER + read_u16(DIGITAL_RADAR_DATA, DRD_RADIAL_LENGTH) as usize;
    let mut block = DIGITAL_RADAR_DATA[start..end].to_vec();
    assert_eq!(&block[..4], b"DCFP");
    block[0] = block_type;
    block[1..4].copy_from_slice(name);
    block
}

fn decode_radar_data(
    data: &[u8],
) -> nexrad_decode::Result<nexrad_decode::messages::digital_radar_data::Message<'_>> {
    let mut messages = decode_messages(data)?;
    assert_eq!(messages.len(), 1);
    match messages.remove(0).into_contents() {
        MessageContents::DigitalRadarData(message) => Ok(*message),
        other => panic!("expected digital radar data, got {other:?}"),
    }
}

#[test]
fn unmodified_fixture_has_no_other_blocks() {
    let message = decode_radar_data(DIGITAL_RADAR_DATA).expect("decodes");
    assert!(message.other_data_blocks().is_empty());
}

#[test]
fn unknown_moment_block_is_kept_and_left_out_of_the_radial() {
    let data = with_extra_block(&renamed_moment_block(b'D', b"XYZ"));
    let message = decode_radar_data(&data).expect("an unknown 'D' block does not fail the decode");

    let others = message.other_data_blocks();
    assert_eq!(others.len(), 1);
    let other = &others[0];
    assert_eq!(other.id().data_block_type(), DataBlockType::DataMoment);
    assert_eq!(other.id().data_block_name_raw(), b"XYZ");

    // The known blocks are unaffected, and the extra block carries the CFP block's gates.
    let cfp = message
        .clutter_filter_power_data_block()
        .expect("CFP block");
    assert_eq!(other.inner().encoded_values(), cfp.inner().encoded_values());
    assert_eq!(
        other.inner().header().data_moment_gate_count(),
        cfp.inner().header().data_moment_gate_count()
    );

    #[cfg(feature = "model")]
    {
        // An unrecognised block has no model product.
        assert!(matches!(
            other.moment_data(),
            Err(Error::UnknownDataBlockType {
                block_type: [b'X', b'Y', b'Z'],
                ..
            })
        ));

        let original = decode_radar_data(DIGITAL_RADAR_DATA).expect("decodes");
        let radial = message.radial().expect("radial");
        assert_eq!(radial, original.radial().expect("radial"));
        assert_eq!(message.into_radial().expect("radial"), radial);
    }
}

#[test]
fn unknown_metadata_block_is_skipped() {
    let mut block = b"RXYZ".to_vec();
    block.extend_from_slice(&[0xAB; 12]);
    let data = with_extra_block(&block);
    let message = decode_radar_data(&data).expect("an unknown 'R' block does not fail the decode");
    assert!(message.other_data_blocks().is_empty());

    let original = decode_radar_data(DIGITAL_RADAR_DATA).expect("decodes");
    assert_eq!(
        message
            .reflectivity_data_block()
            .map(|b| b.inner().encoded_values()),
        original
            .reflectivity_data_block()
            .map(|b| b.inner().encoded_values())
    );
    #[cfg(feature = "model")]
    assert_eq!(
        message.radial().expect("radial"),
        original.radial().expect("radial")
    );
}

#[test]
fn unknown_block_between_known_blocks_is_skipped() {
    // Renaming the fixture's VOL block leaves an unknown 'R' block followed by known blocks.
    let block = DRD_HEADER + pointers(DIGITAL_RADAR_DATA)[0];
    let mut data = DIGITAL_RADAR_DATA.to_vec();
    assert_eq!(&data[block..block + 4], b"RVOL");
    data[block + 1..block + 4].copy_from_slice(b"XYZ");

    let message = decode_radar_data(&data).expect("decodes");
    assert!(message.volume_data_block().is_none());
    assert!(message.elevation_data_block().is_some());
    assert!(message.reflectivity_data_block().is_some());
}

#[test]
fn unknown_block_type_byte_is_an_error() {
    let data = with_extra_block(&renamed_moment_block(b'Q', b"XYZ"));
    let block_offset = DRD_HEADER + *pointers(&data).last().expect("a block");
    assert_eq!(
        decode_messages(&data),
        Err(Error::UnknownDataBlockType {
            block_type: *b"XYZ",
            offset: block_offset,
        })
    );
}

#[test]
fn owned_message_keeps_other_blocks() {
    let data = with_extra_block(&renamed_moment_block(b'D', b"XYZ"));
    let message = decode_radar_data(&data).expect("decodes").into_owned();
    assert_eq!(message.other_data_blocks().len(), 1);
}
