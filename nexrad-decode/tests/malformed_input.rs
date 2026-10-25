//! Negative tests for malformed input handling.
//!
//! These tests verify that the decode functions report invalid, truncated, and corrupted input
//! as errors at the right offset, and never panic.

use nexrad_decode::messages::MessageHeader;
use nexrad_decode::{decode_messages, iter_messages, Error, MessageContents, MessageType};

const RDA_STATUS: &[u8] = include_bytes!("data/messages/rda_status_data.bin");
const VOLUME_COVERAGE_PATTERN: &[u8] = include_bytes!("data/messages/volume_coverage_pattern.bin");
const DIGITAL_RADAR_DATA: &[u8] = include_bytes!("data/messages/digital_radar_data_full.bin");

/// Offset of the digital radar data header within a type 31 message.
const DRD_HEADER: usize = MessageHeader::SIZE;
/// Offset of the first data block pointer within a type 31 message.
const DRD_POINTERS: usize = DRD_HEADER + 32;

fn concat(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}

/// The first data block pointer of the digital radar data fixture, relative to its header.
fn first_block_pointer() -> usize {
    let bytes = &DIGITAL_RADAR_DATA[DRD_POINTERS..DRD_POINTERS + 4];
    u32::from_be_bytes(bytes.try_into().expect("four bytes")) as usize
}

#[test]
fn test_empty_input() {
    assert_eq!(decode_messages(&[]), Ok(Vec::new()));
}

#[test]
fn test_zero_tail_shorter_than_header_is_padding() {
    assert_eq!(decode_messages(&[0]), Ok(Vec::new()));
    assert_eq!(
        decode_messages(&[0; MessageHeader::SIZE - 1]),
        Ok(Vec::new())
    );

    let data = concat(&[RDA_STATUS, &[0; MessageHeader::SIZE - 1]]);
    let messages = decode_messages(&data).expect("decodes");
    assert_eq!(messages.len(), 1);

    let mut iter = iter_messages(&data);
    assert!(matches!(iter.next(), Some(Ok(_))));
    assert!(iter.next().is_none());
    assert_eq!(iter.offset(), data.len());
}

#[test]
fn test_truncated_header() {
    let data = concat(&[RDA_STATUS, &[0xFF; 10]]);
    assert_eq!(
        decode_messages(&data),
        Err(Error::UnexpectedEof {
            offset: RDA_STATUS.len()
        })
    );

    // A short tail that is not all zero is an error even at the start of the input.
    assert_eq!(
        decode_messages(&[0, 0, 1]),
        Err(Error::UnexpectedEof { offset: 0 })
    );
}

#[test]
fn test_zero_tail_longer_than_header_is_an_error() {
    // A zero header declares a fixed-segment frame, which must be present in full.
    let data = concat(&[RDA_STATUS, &[0; MessageHeader::SIZE]]);
    assert_eq!(
        decode_messages(&data),
        Err(Error::UnexpectedEof {
            offset: RDA_STATUS.len()
        })
    );
}

#[test]
fn test_truncated_fixed_segment_body() {
    let data = concat(&[RDA_STATUS, &VOLUME_COVERAGE_PATTERN[..1000]]);
    assert_eq!(
        decode_messages(&data),
        Err(Error::UnexpectedEof {
            offset: RDA_STATUS.len()
        })
    );
}

#[test]
fn test_truncated_digital_radar_data_body() {
    let data = concat(&[RDA_STATUS, &DIGITAL_RADAR_DATA[..DRD_HEADER + 10]]);
    assert_eq!(
        decode_messages(&data),
        Err(Error::UnexpectedEof {
            offset: RDA_STATUS.len() + DRD_HEADER
        })
    );
}

#[test]
fn test_bad_data_block_pointer() {
    let mut radar_data = DIGITAL_RADAR_DATA.to_vec();
    radar_data[DRD_POINTERS..DRD_POINTERS + 4].copy_from_slice(&0u32.to_be_bytes());
    let data = concat(&[RDA_STATUS, &radar_data]);

    // The pointer table is 8 pointers long, so the first block cannot start before byte 64.
    let pointers_end = DRD_POINTERS - DRD_HEADER + 8 * 4;
    assert_eq!(
        decode_messages(&data),
        Err(Error::InvalidDataBlockPointer {
            bytes: pointers_end,
            position: RDA_STATUS.len() + DRD_HEADER + pointers_end,
        })
    );
}

#[test]
fn test_unknown_data_block_name() {
    let block = DRD_HEADER + first_block_pointer();
    let mut radar_data = DIGITAL_RADAR_DATA.to_vec();
    assert_eq!(&radar_data[block + 1..block + 4], b"VOL");
    radar_data[block + 1..block + 4].copy_from_slice(b"XYZ");
    let data = concat(&[RDA_STATUS, &radar_data]);

    assert_eq!(
        decode_messages(&data),
        Err(Error::UnknownDataBlockType {
            block_type: *b"XYZ",
            offset: RDA_STATUS.len() + block,
        })
    );
}

#[test]
fn test_iter_messages_yields_messages_before_the_error() {
    let mut radar_data = DIGITAL_RADAR_DATA.to_vec();
    radar_data[DRD_POINTERS..DRD_POINTERS + 4].copy_from_slice(&0u32.to_be_bytes());
    let data = concat(&[
        RDA_STATUS,
        VOLUME_COVERAGE_PATTERN,
        &radar_data,
        DIGITAL_RADAR_DATA,
    ]);
    let error_offset = RDA_STATUS.len() + VOLUME_COVERAGE_PATTERN.len();

    let mut iter = iter_messages(&data);

    let status = iter.next().expect("item").expect("decodes");
    assert_eq!(status.header().message_type(), MessageType::RdaStatusData);
    assert_eq!(iter.offset(), RDA_STATUS.len());

    let vcp = iter.next().expect("item").expect("decodes");
    assert_eq!(
        vcp.header().message_type(),
        MessageType::RdaVolumeCoveragePattern
    );

    assert!(matches!(
        iter.next(),
        Some(Err(Error::InvalidDataBlockPointer { .. }))
    ));
    assert_eq!(iter.offset(), error_offset);

    // The iterator is finished after an error, even though a valid message follows.
    assert!(iter.next().is_none());
    assert!(iter.next().is_none());
    assert_eq!(iter.offset(), error_offset);
}

#[test]
fn test_digital_radar_data_padding_is_skipped() {
    // TDWR radials end a few bytes after their last data block; the declared message size
    // counts that padding.
    const PADDING: usize = 6;
    let mut padded = DIGITAL_RADAR_DATA.to_vec();
    let halfwords = u16::from_be_bytes([padded[12], padded[13]]) + (PADDING / 2) as u16;
    padded[12..14].copy_from_slice(&halfwords.to_be_bytes());
    padded.extend_from_slice(&[0; PADDING]);

    let data = concat(&[&padded, &padded, RDA_STATUS]);
    let messages = decode_messages(&data).expect("decodes");
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].size_bytes(), padded.len());
    assert_eq!(messages[1].offset_bytes(), padded.len());
    assert_eq!(
        messages[2].header().message_type(),
        MessageType::RdaStatusData
    );
}

#[test]
fn test_iter_messages_matches_decode_messages() {
    let data = concat(&[RDA_STATUS, VOLUME_COVERAGE_PATTERN, DIGITAL_RADAR_DATA]);
    let iterated = iter_messages(&data)
        .collect::<Result<Vec<_>, _>>()
        .expect("decodes");
    assert_eq!(iterated, decode_messages(&data).expect("decodes"));
    assert_eq!(iterated.len(), 3);
}

#[test]
fn test_into_owned_outlives_input() {
    let buffer = concat(&[RDA_STATUS, DIGITAL_RADAR_DATA]);
    let owned: Vec<_> = decode_messages(&buffer)
        .expect("decodes")
        .into_iter()
        .map(|message| message.into_owned())
        .collect();
    drop(buffer);

    let expected = decode_messages(&concat(&[RDA_STATUS, DIGITAL_RADAR_DATA]))
        .expect("decodes")
        .into_iter()
        .map(|message| message.into_owned())
        .collect::<Vec<_>>();
    assert_eq!(owned, expected);

    let MessageContents::DigitalRadarData(radar_data) = owned[1].contents() else {
        panic!("expected digital radar data");
    };
    assert_eq!(owned[1].offset_bytes(), RDA_STATUS.len());
    assert_eq!(radar_data.header().radar_identifier(), Some("KDMX"));
    let reflectivity = radar_data
        .reflectivity_data_block()
        .expect("reflectivity block");
    assert!(!reflectivity.inner().encoded_values().is_empty());
}

#[test]
fn test_other_carries_type_and_payload_of_fixed_segment_message() {
    // An RDA status frame relabelled with a message type number the ICD does not define.
    let mut frame = RDA_STATUS.to_vec();
    frame[15] = 40;
    let header = MessageHeader::parse(&frame).expect("parses");
    let payload_size = header.message_size_bytes() as usize - (MessageHeader::SIZE - 12);

    let messages = decode_messages(&frame).expect("decodes");
    assert_eq!(messages.len(), 1);
    let contents = messages[0].contents();
    assert_eq!(contents.message_type(), MessageType::Unknown(40));
    match contents {
        MessageContents::Other { message_type, data } => {
            assert_eq!(*message_type, MessageType::Unknown(40));
            assert_eq!(
                data.as_ref(),
                &frame[MessageHeader::SIZE..MessageHeader::SIZE + payload_size]
            );
        }
        other => panic!("expected Other, got {other:?}"),
    }
}

#[test]
fn test_other_carries_type_and_payload_of_variable_length_message() {
    let payload = b"0123456789";
    let mut message = vec![0u8; MessageHeader::SIZE];
    message[12..14].copy_from_slice(&0xFFFFu16.to_be_bytes());
    message[14] = 8;
    message[15] = 40;
    // The message size counts the 16-byte message header and the payload.
    message[24..28].copy_from_slice(&(16 + payload.len() as u32).to_be_bytes());
    message.extend_from_slice(payload);

    let messages = decode_messages(&message).expect("decodes");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].size_bytes(), message.len());
    assert_eq!(
        messages[0].contents(),
        &MessageContents::Other {
            message_type: MessageType::Unknown(40),
            data: payload.as_slice().into(),
        }
    );

    // A payload cut short is an error at the message's offset.
    assert_eq!(
        decode_messages(&message[..message.len() - 1]),
        Err(Error::UnexpectedEof { offset: 0 })
    );
}

#[test]
fn test_random_garbage_data() {
    // Random-looking data that doesn't conform to any format
    let garbage: Vec<u8> = (0..1000).map(|i| (i * 17 % 256) as u8).collect();
    // Should not panic regardless of input
    let _ = decode_messages(&garbage);
}

#[test]
fn test_all_zeros() {
    assert_eq!(
        decode_messages(&[0; 1000]),
        Err(Error::UnexpectedEof { offset: 0 })
    );
}

#[test]
fn test_all_ones() {
    let _ = decode_messages(&[0xFF; 1000]);
}

#[test]
fn test_repeating_pattern() {
    let pattern: Vec<u8> = (0..1000).map(|i| (i % 16) as u8).collect();
    let _ = decode_messages(&pattern);
}

/// Test that real data truncated inside its first message is an error
#[test]
fn test_truncated_real_data() {
    for truncate_at in [30, 50, 100, 500, DIGITAL_RADAR_DATA.len() / 2] {
        let truncated = &DIGITAL_RADAR_DATA[..truncate_at];
        assert!(decode_messages(truncated).is_err(), "{truncate_at}");
    }
}

/// Test invalid redundant channel values
#[test]
fn test_invalid_redundant_channel() {
    let mut frame = RDA_STATUS.to_vec();
    frame[14] = 99;
    let messages = decode_messages(&frame).expect("decodes");
    assert_eq!(messages.len(), 1);
}

/// Test that the decoder handles minimal sized inputs at boundaries
#[test]
fn test_boundary_sizes() {
    for size in 26..32 {
        let _ = decode_messages(&vec![0u8; size]);
    }
    for size in 2430..2440 {
        let _ = decode_messages(&vec![0u8; size]);
    }
}

/// Test with oversized message length field
#[test]
fn test_oversized_message_length() {
    let mut data = vec![0u8; 100];
    data[12] = 0xFF;
    data[13] = 0xFF;
    data[24] = 0xFF;
    assert_eq!(
        decode_messages(&data),
        Err(Error::UnexpectedEof { offset: 0 })
    );
}
