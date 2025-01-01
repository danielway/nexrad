//!
//! Message type 15 "Clutter Filter Map" contains information about clutter filter maps that are
//! used to filter clutter from radar products. The clutter filter map is a 3D array of elevation,
//! azimuth, and range zones that define the clutter filter behavior for radar products.
//!

mod header;
pub use header::Header;

mod message;
pub use message::Message;

mod segment;
pub use segment::Segment;

mod elevation_segment;
pub use elevation_segment::ElevationSegment;

mod azimuth_segment;
pub use azimuth_segment::{AzimuthSegment, AzimuthSegmentHeader};

mod range_zone;
pub use range_zone::RangeZone;

mod definitions;
pub use definitions::*;

use crate::messages::decode_message_header;
use crate::result::Error::MessageSegmentationError;
use crate::result::Result;
use crate::util::deserialize;
use std::io::Read;

/// Decodes a clutter filter map message type 15 from the provided reader.
pub fn decode_clutter_filter_map<R: Read>(reader: &mut R) -> Result<Message> {
    let mut message_header = decode_message_header(reader)?;
    debug_assert!(message_header.segmented());
    debug_assert_eq!(message_header.segment_number(), Some(1));

    let header: Header = deserialize(reader)?;
    let elevation_segment_count = header.elevation_segment_count as u8;

    let mut message = Message::new(header);

    // message segment 1
    //
    // 0x0  	0xC 	RPG prefix, 6 HW
    //
    // 0xC 	    0x1C 	Message header (segment size 1208 HW)
    // 0x1C 	0x97C 	Message contents
    // 0x1C 	0x22 		Clutter map header
    // 0x22 	0x97C 		Clutter map data
    // 0x22 	0x24 			Segment header: zone count = 1
    // 0x24 	0x28 			Range zone: opcode = 1 (map in ctrl), end range = 511
    //
    // 0x97C 	0x980 	Empty HW, unknown - trailing 2 that don't fit bytes divisible by 3
    // 					    how do we compute expected trailing?
    //
    // message segment 2
    //
    // 0x980 	0x98C 	RPG prefix, 6 HW
    //
    // 0x98C 	0x99C 	Message header (start of M1S1 0x1C + segment
    // 					    size 1208 HW = this position)
    // 0x99C 	0x130C 	Message contents
    //
    // 0x12FC 	0x1300 	Empty HW, unknown - trailing
    //
    // message segment 3
    //
    // 0x1300 	0x130C 	RPG prefix, 6 HW
    //
    // 0x130C 	0x131C 	Message header

    loop {
        let segment_number = message_header
            .segment_number()
            .ok_or(MessageSegmentationError)?;
        let mut message_segment = Segment::new(segment_number);

        for elevation_segment_number in 0..elevation_segment_count {
            let mut elevation_segment = ElevationSegment::new(elevation_segment_number);

            for azimuth_number in 0..360 {
                let azimuth_segment_header: AzimuthSegmentHeader = deserialize(reader)?;
                let range_zone_count = azimuth_segment_header.range_zone_count as usize;

                let mut azimuth_segment =
                    AzimuthSegment::new(azimuth_segment_header, azimuth_number);
                for _ in 0..range_zone_count {
                    let range_zone: RangeZone = deserialize(reader)?;
                    azimuth_segment.range_zones.push(range_zone);
                }

                elevation_segment.azimuth_segments.push(azimuth_segment);
            }

            message_segment.elevation_segments.push(elevation_segment);
        }

        message.segments.push(message_segment);

        if message_header.segment_number() == message_header.segment_count() {
            break;
        }

        message_header = decode_message_header(reader)?;
    }

    Ok(message)
}
