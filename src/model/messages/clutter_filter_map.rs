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
