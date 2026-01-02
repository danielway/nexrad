//!
//! Message type 15 "Clutter Filter Map" contains information about clutter filter maps that are
//! used to filter clutter from radar products. The clutter filter map is a 3D array of elevation,
//! azimuth, and range zones that define the clutter filter behavior for radar products.
//!

mod azimuth_segment;
pub use azimuth_segment::AzimuthSegment;

mod elevation_segment;
pub use elevation_segment::ElevationSegment;

mod message;
pub use message::Message;

mod raw;
pub use raw::*;
