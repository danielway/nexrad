//!
//! Message type 5 "Volume Coverage Pattern" consists of base VCP information such as
//! the number of elevation cuts and metadata about the entire volume, as well as a data
//! block for each elevation with metadata about that specific cut such as the waveform
//! type and other metadata.
//!

mod definitions;
pub use definitions::*;

mod header;
use std::io::Read;

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
