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

mod data_block;
pub use data_block::DataBlock;

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

use crate::result::{Error, Result};
use crate::util::deserialize;
use std::io::{self, Read};

const HEADER_SIZE: usize = size_of::<Header>();
const POINTER_SIZE: usize = size_of::<u32>();
const BLOCK_ID_SIZE: usize = size_of::<DataBlockId>();
const VOLUME_DATA_BLOCK_SIZE: usize = size_of::<VolumeDataBlock>();
const ELEVATION_DATA_BLOCK_SIZE: usize = size_of::<ElevationDataBlock>();
const RADIAL_DATA_BLOCK_SIZE: usize = size_of::<RadialDataBlock>();
const GENERIC_DATA_HEADER_SIZE: usize = size_of::<GenericDataBlockHeader>();

/// Decodes a digital radar data message type 31 from the provided reader.
pub fn decode_digital_radar_data<R: Read>(reader: &mut R) -> Result<Message> {
    let header: Header = deserialize(reader)?;
    let mut message = Message::new(header);

    let pointers_space = message.header.data_block_count as usize * POINTER_SIZE;
    let mut pointers_raw = vec![0; pointers_space];
    reader.read_exact(&mut pointers_raw)?;

    let pointers = pointers_raw
        .chunks_exact(size_of::<u32>())
        .map(|v| {
            v.try_into()
                .map_err(|_| Error::DecodingError("message pointers".to_string()))
                .map(u32::from_be_bytes)
        })
        .collect::<Result<Vec<_>>>()?;

    let mut read_position = HEADER_SIZE + pointers_raw.len();
    for pointer in pointers {
        if (pointer as usize) < read_position {
            panic!("Cannot read backwards")
        }

        let pointer_difference = pointer as usize - read_position;
        if pointer_difference > 0 {
            let mut take_reader = reader.take(pointer_difference as u64);
            io::copy(&mut take_reader, &mut io::sink())?;
            read_position += pointer_difference;
        }

        let data_block_id: DataBlockId = deserialize(reader)?;
        read_position += BLOCK_ID_SIZE;

        let data_block_name = data_block_id.data_block_name();
        match data_block_name.as_str() {
            "VOL" => {
                message.volume_data_block =
                    Some(DataBlock::new(data_block_id, deserialize(reader)?));
                read_position += VOLUME_DATA_BLOCK_SIZE;
            }
            "ELV" => {
                message.elevation_data_block =
                    Some(DataBlock::new(data_block_id, deserialize(reader)?));
                read_position += ELEVATION_DATA_BLOCK_SIZE;
            }
            "RAD" => {
                message.radial_data_block =
                    Some(DataBlock::new(data_block_id, deserialize(reader)?));
                read_position += RADIAL_DATA_BLOCK_SIZE;
            }
            _ => {
                let generic_header: GenericDataBlockHeader = deserialize(reader)?;
                read_position += GENERIC_DATA_HEADER_SIZE;

                let mut generic_data_block =
                    DataBlock::new(data_block_id, GenericDataBlock::new(generic_header));
                reader.read_exact(&mut generic_data_block.data.encoded_data)?;
                read_position += generic_data_block.data.encoded_data.len();

                match data_block_name.as_str() {
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
                    _ => panic!("Unknown generic data block type: {}", data_block_name),
                }
            }
        }
    }

    Ok(message)
}
