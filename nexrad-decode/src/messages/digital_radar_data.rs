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

use crate::result::{Error, Result};
use std::io::{Read, Seek, SeekFrom};

/// Decodes a digital radar data message type 31 from the provided reader.
pub fn decode_digital_radar_data<R: Read + Seek>(reader: &mut R) -> Result<Message> {
    let start_position = reader.stream_position()?;

    // Read and decode header
    let mut header_bytes = vec![0u8; size_of::<Header>()];
    reader.read_exact(&mut header_bytes)?;
    let (header, _) = Header::decode_ref(&header_bytes)?;
    let mut message = Message::new(header.clone());

    let pointers_space = message.header.data_block_count.get() as usize * size_of::<u32>();
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

    for pointer in pointers {
        reader.seek(SeekFrom::Start(start_position + pointer as u64))?;

        // Read data block ID
        let mut id_bytes = [0u8; size_of::<DataBlockId>()];
        reader.read_exact(&mut id_bytes)?;
        let (data_block_id, _) = DataBlockId::decode_ref(&id_bytes)?;

        // Seek back to start of block
        reader.seek(SeekFrom::Current(-(size_of::<DataBlockId>() as i64)))?;

        match data_block_id.data_block_name().as_str() {
            "VOL" => {
                let mut block_bytes = vec![0u8; size_of::<VolumeDataBlock>()];
                reader.read_exact(&mut block_bytes)?;
                let (block, _) = VolumeDataBlock::decode_ref(&block_bytes)?;
                message.volume_data_block = Some(block.clone());
            }
            "ELV" => {
                let mut block_bytes = vec![0u8; size_of::<ElevationDataBlock>()];
                reader.read_exact(&mut block_bytes)?;
                let (block, _) = ElevationDataBlock::decode_ref(&block_bytes)?;
                message.elevation_data_block = Some(block.clone());
            }
            "RAD" => {
                let mut block_bytes = vec![0u8; size_of::<RadialDataBlock>()];
                reader.read_exact(&mut block_bytes)?;
                let (block, _) = RadialDataBlock::decode_ref(&block_bytes)?;
                message.radial_data_block = Some(block.clone());
            }
            _ => {
                let mut header_bytes = vec![0u8; size_of::<GenericDataBlockHeader>()];
                reader.read_exact(&mut header_bytes)?;
                let (generic_header, _) = GenericDataBlockHeader::decode_ref(&header_bytes)?;

                let mut generic_data_block = GenericDataBlock::new(generic_header.clone());
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
                    _ => panic!("Unknown generic data block type: {data_block_id:?}"),
                }
            }
        }
    }

    Ok(message)
}
