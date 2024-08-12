//!
//! Decoding functions for NEXRAD Archive II radar data files.
//!

use crate::messages::clutter_filter_map;
use crate::messages::clutter_filter_map::ElevationSegment;
use crate::messages::digital_radar_data;
use crate::messages::digital_radar_data::{DataBlockId, GenericDataBlock, GenericDataBlockHeader};
use crate::messages::message_header::MessageHeader;
use crate::messages::rda_status_data;
use crate::result::Result;
use crate::Archive2Header;
use bincode::{DefaultOptions, Options};
use serde::de::DeserializeOwned;
use std::io::{Read, Seek, SeekFrom};
use std::mem::size_of;

/// Decodes an Archive II header from the provided reader.
pub fn decode_archive2_header<R: Read>(reader: &mut R) -> Result<Archive2Header> {
    deserialize(reader)
}

/// Decodes a message header from the provided reader.
pub fn decode_message_header<R: Read>(reader: &mut R) -> Result<MessageHeader> {
    deserialize(reader)
}

/// Decodes an RDA status message type 2 from the provided reader.
pub fn decode_rda_status_message<R: Read>(reader: &mut R) -> Result<rda_status_data::Message> {
    deserialize(reader)
}

/// Decodes a digital radar data message type 31 from the provided reader.
pub fn decode_digital_radar_data<R: Read + Seek>(
    reader: &mut R,
) -> Result<digital_radar_data::Message> {
    let start_position = reader.stream_position()?;

    let header = deserialize(reader)?;
    let mut message = digital_radar_data::Message::new(header);

    let pointers_space = message.header.data_block_count as usize * size_of::<u32>();
    let mut pointers_raw = vec![0; pointers_space];
    reader.read_exact(&mut pointers_raw).unwrap();

    let pointers = pointers_raw
        .chunks_exact(size_of::<u32>())
        .map(|v| <u32>::from_be_bytes(v.try_into().unwrap()))
        .collect::<Vec<_>>();

    for pointer in pointers {
        reader.seek(SeekFrom::Start(start_position + pointer as u64))?;

        let data_block_id: DataBlockId = deserialize(reader)?;
        reader.seek(SeekFrom::Current(-4))?;

        match data_block_id.data_block_name().as_str() {
            "VOL" => {
                message.volume_data_block = Some(deserialize(reader)?);
            }
            "ELV" => {
                message.elevation_data_block = Some(deserialize(reader)?);
            }
            "RAD" => {
                message.radial_data_block = Some(deserialize(reader)?);
            }
            _ => {
                let generic_header: GenericDataBlockHeader = deserialize(reader)?;

                let mut generic_data_block = GenericDataBlock::new(generic_header);
                reader.read_exact(&mut generic_data_block.encoded_data)?;

                match data_block_id.data_block_name().as_str() {
                    "REF" => {
                        message.reflectivity_data_block = Some(generic_data_block);
                    }
                    "VEL" => {
                        message.velocity_data_block = Some(generic_data_block);
                    }
                    "SW " => {
                        message.spectrum_width_data_block = Some(generic_data_block);
                    }
                    "ZDR" => {
                        message.differential_reflectivity_data_block = Some(generic_data_block);
                    }
                    "PHI" => {
                        message.differential_phase_data_block = Some(generic_data_block);
                    }
                    "RHO" => {
                        message.correlation_coefficient_data_block = Some(generic_data_block);
                    }
                    "CFP" => {
                        message.specific_diff_phase_data_block = Some(generic_data_block);
                    }
                    _ => panic!("Unknown generic data block type: {:?}", data_block_id),
                }
            }
        }
    }

    Ok(message)
}

/// Decodes a clutter filter map message type 15 from the provided reader.
pub fn decode_clutter_filter_map<R: Read>(reader: &mut R) -> Result<clutter_filter_map::Message> {
    let header: clutter_filter_map::Header = deserialize(reader)?;
    let elevation_segment_count = header.elevation_segment_count as u8;

    let mut message = clutter_filter_map::Message::new(header);

    for elevation_segment_number in 0..elevation_segment_count {
        let mut elevation_segment = ElevationSegment::new(elevation_segment_number);

        for azimuth_number in 0..360 {
            let azimuth_segment_header: clutter_filter_map::AzimuthSegmentHeader =
                deserialize(reader)?;
            let range_zone_count = azimuth_segment_header.range_zone_count as usize;

            let mut azimuth_segment =
                clutter_filter_map::AzimuthSegment::new(azimuth_segment_header, azimuth_number);
            for _ in 0..range_zone_count {
                azimuth_segment.range_zones.push(deserialize(reader)?);
            }

            elevation_segment.azimuth_segments.push(azimuth_segment);
        }

        message.elevation_segments.push(elevation_segment);
    }

    Ok(message)
}

/// Attempts to deserialize some struct from the provided binary reader.
fn deserialize<R: Read, S: DeserializeOwned>(reader: &mut R) -> Result<S> {
    Ok(DefaultOptions::new()
        .with_fixint_encoding()
        .with_big_endian()
        .deserialize_from(reader.by_ref())?)
}
