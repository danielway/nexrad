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
use crate::util::deserialize;

/// Decodes a volume coverage pattern message type 5 from the provided reader.
pub fn decode_volume_coverage_pattern<R: Read>(reader: &mut R) -> Result<Message> {
    let header: Header = deserialize(reader)?;

    let mut elevations: Vec<ElevationDataBlock> = Vec::new();
    for _ in 0..header.number_of_elevation_cuts {
        elevations.push(deserialize(reader)?);
    }

    let message = Message::new(header, elevations);

    Ok(message)
}
