//!
//! Message type 15 "Clutter Filter Map" contains information about clutter filter maps that are
//! used to filter clutter from radar products. The clutter filter map is a 3D array of elevation,
//! azimuth, and range zones that define the clutter filter behavior for radar products.
//!

mod header;
pub use header::Header;

mod message;
pub use message::Message;

mod elevation_segment;
pub use elevation_segment::ElevationSegment;

mod azimuth_segment;
pub use azimuth_segment::{AzimuthSegment, AzimuthSegmentHeader};

mod range_zone;
pub use range_zone::RangeZone;

mod definitions;
pub use definitions::*;

use crate::result::Result;
use crate::util::deserialize;
use std::io::Read;

/// Decodes a clutter filter map message type 15 from the provided reader.
pub fn decode_clutter_filter_map<R: Read>(reader: &mut R) -> Result<Message> {
    let header: Header = deserialize(reader)?;
    let elevation_segment_count = header.elevation_segment_count as u8;

    let mut message = Message::new(header);

    for elevation_segment_number in 0..elevation_segment_count {
        let mut elevation_segment = ElevationSegment::new(elevation_segment_number);

        for azimuth_number in 0..360 {
            let azimuth_segment_header: AzimuthSegmentHeader = deserialize(reader)?;
            let range_zone_count = azimuth_segment_header.range_zone_count as usize;

            let mut azimuth_segment = AzimuthSegment::new(azimuth_segment_header, azimuth_number);
            for _ in 0..range_zone_count {
                azimuth_segment.range_zones.push(deserialize(reader)?);
            }

            elevation_segment.azimuth_segments.push(azimuth_segment);
        }

        message.elevation_segments.push(elevation_segment);
    }

    Ok(message)
}
