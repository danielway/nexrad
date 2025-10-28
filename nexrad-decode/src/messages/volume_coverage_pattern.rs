//!
//! Message type 5 "Volume Coverage Pattern" provides details about the volume
//! coverage pattern being used. The RDA sends the Volume Coverage Pattern message
//! upon wideband connection and at the beginning of each volume scan. The volume
//! coverage pattern message includes a header which describes how the volume is being
//! collected as well as a block for each elevation cut detailing the radar settings
//! being used for that cut.
//!

use std::io::Read;

mod definitions;
pub use definitions::*;

mod header;
pub use header::Header;

mod message;
pub use message::Message;

mod elevation_data_block;
pub use elevation_data_block::ElevationDataBlock;

use crate::result::Result;

/// Decodes a volume coverage pattern message type 5 from the provided reader.
pub fn decode_volume_coverage_pattern<R: Read>(reader: &mut R) -> Result<Message> {
    let mut header_bytes = vec![0u8; size_of::<Header>()];
    reader.read_exact(&mut header_bytes)?;
    let (header, _) = Header::decode_ref(&header_bytes)?;

    let mut elevations: Vec<ElevationDataBlock> = Vec::new();
    for _ in 0..header.number_of_elevation_cuts.get() {
        let mut elevation_bytes = vec![0u8; size_of::<ElevationDataBlock>()];
        reader.read_exact(&mut elevation_bytes)?;
        let (elevation, _) = ElevationDataBlock::decode_ref(&elevation_bytes)?;
        elevations.push(elevation.clone());
    }

    let message = Message::new(header.clone(), elevations);

    Ok(message)
}
