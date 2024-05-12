//!
//! TODO
//!

use crate::model::messages::clutter_filter_map;
use crate::model::messages::clutter_filter_map::ElevationSegment;
use crate::model::messages::digital_radar_data;
use crate::model::messages::digital_radar_data::{
    DataMomentGenericPointerType, DataMomentPointerType, GenericDataBlock,
};
use crate::model::messages::message_header::MessageHeader;
use crate::model::messages::rda_status_data;
use crate::model::Archive2Header;
use crate::result::Result;
use bincode::{DefaultOptions, Options};
use serde::de::DeserializeOwned;
use std::io::{Read, Seek, SeekFrom};

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
    let seek_to_pointer = |reader: &mut R, pointer: u32| -> Result<()> {
        if pointer as u64 != reader.stream_position()? {
            reader.seek(SeekFrom::Start(start_position + pointer as u64))?;
        }
        Ok(())
    };

    let header = deserialize(reader)?;
    let mut message = digital_radar_data::Message::new(header);
    for pointer in message.header.pointers() {
        match pointer.data_moment_type {
            DataMomentPointerType::Volume => {
                seek_to_pointer(reader, pointer.pointer)?;
                message.volume_data_block = Some(deserialize(reader)?);
            }
            DataMomentPointerType::Elevation => {
                seek_to_pointer(reader, pointer.pointer)?;
                message.elevation_data_block = Some(deserialize(reader)?);
            }
            DataMomentPointerType::Radial => {
                seek_to_pointer(reader, pointer.pointer)?;
                message.radial_data_block = Some(deserialize(reader)?);
            }
            DataMomentPointerType::Generic(generic_type) => {
                seek_to_pointer(reader, pointer.pointer)?;
                let generic_header = deserialize(reader)?;

                let mut generic_data_block = GenericDataBlock::new(generic_header);
                reader.read_exact(&mut generic_data_block.data)?;

                match generic_type {
                    DataMomentGenericPointerType::Reflectivity => {
                        message.reflectivity_data_block = Some(generic_data_block);
                    }
                    DataMomentGenericPointerType::Velocity => {
                        message.velocity_data_block = Some(generic_data_block);
                    }
                    DataMomentGenericPointerType::SpectrumWidth => {
                        message.spectrum_width_data_block = Some(generic_data_block);
                    }
                    DataMomentGenericPointerType::DifferentialReflectivity => {
                        message.differential_reflectivity_data_block = Some(generic_data_block);
                    }
                    DataMomentGenericPointerType::DifferentialPhase => {
                        message.differential_phase_data_block = Some(generic_data_block);
                    }
                    DataMomentGenericPointerType::CorrelationCoefficient => {
                        message.correlation_coefficient_data_block = Some(generic_data_block);
                    }
                    DataMomentGenericPointerType::SpecificDiffPhase => {
                        message.specific_diff_phase_data_block = Some(generic_data_block);
                    }
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
