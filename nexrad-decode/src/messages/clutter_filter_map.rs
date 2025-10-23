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
use std::io::Read;

/// Decodes a clutter filter map message type 15 from the provided reader.
pub fn decode_clutter_filter_map<R: Read>(reader: &mut R) -> Result<Message> {
    let mut header_bytes = vec![0u8; size_of::<Header>()];
    reader.read_exact(&mut header_bytes)?;
    let (header, _) = Header::decode_ref(&header_bytes)?;
    let elevation_segment_count = header.elevation_segment_count.get() as u8;

    let mut message = Message::new(header.clone());

    for elevation_segment_number in 0..elevation_segment_count {
        let mut elevation_segment = ElevationSegment::new(elevation_segment_number);

        for azimuth_number in 0..360 {
            let mut azimuth_header_bytes = vec![0u8; size_of::<AzimuthSegmentHeader>()];
            reader.read_exact(&mut azimuth_header_bytes)?;
            let (azimuth_segment_header, _) =
                AzimuthSegmentHeader::decode_ref(&azimuth_header_bytes)?;
            let range_zone_count = azimuth_segment_header.range_zone_count.get() as usize;

            let mut azimuth_segment =
                AzimuthSegment::new(azimuth_segment_header.clone(), azimuth_number);
            for _ in 0..range_zone_count {
                let mut range_zone_bytes = vec![0u8; size_of::<RangeZone>()];
                reader.read_exact(&mut range_zone_bytes)?;
                let (range_zone, _) = RangeZone::decode_ref(&range_zone_bytes)?;
                azimuth_segment.range_zones.push(range_zone.clone());
            }

            elevation_segment.azimuth_segments.push(azimuth_segment);
        }

        message.elevation_segments.push(elevation_segment);
    }

    Ok(message)
}
