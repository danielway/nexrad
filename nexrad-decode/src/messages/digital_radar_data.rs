//!
//! Message type 31 "Digital Radar Data" consists of base data information such as reflectivity,
//! mean radial velocity, spectrum width, differential reflectivity, differential phase, correlation
//! coefficient, azimuth angle, elevation angle, cut type, scanning strategy, and calibration
//! parameters. The frequency and volume of the message is dependent on the scanning strategy and
//! the type of data associated with that strategy.
//!

mod header;
pub use header::Header;

mod message;
pub use message::Message;

mod data_block_id;
pub use data_block_id::DataBlockId;

mod volume_data_block;
pub use volume_data_block::VolumeDataBlock;

mod generic_data_block;
pub use generic_data_block::{GenericDataBlock, GenericDataBlockHeader};

mod elevation_data_block;
pub use elevation_data_block::ElevationDataBlock;

mod radial_data_block;
pub use radial_data_block::RadialDataBlock;

mod definitions;
pub use definitions::*;

mod spot_blanking_status;
pub use spot_blanking_status::*;

mod pointers;
pub use pointers::*;

use crate::result::Result;
use crate::util::deserialize;
use std::io::{Read, Seek, SeekFrom};

/// Decodes a digital radar data message type 31 from the provided reader.
pub fn decode_digital_radar_data<R: Read + Seek>(reader: &mut R) -> Result<Message> {
    let start_position = reader.stream_position()?;

    let header = deserialize(reader)?;
    let mut message = Message::new(header);

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
