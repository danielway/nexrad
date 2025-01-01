use crate::messages::clutter_filter_map::elevation_segment::ElevationSegment;
use std::fmt::Debug;

/// A clutter filter map segment describing elevations, azimuths, and ranges containing clutter to
/// filtered from radar products.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Segment {
    /// This segment's message in the message.
    pub segment_number: u16,

    /// The elevation segments defined in this clutter filter map.
    pub elevation_segments: Vec<ElevationSegment>,
}

impl Segment {
    /// Creates a new clutter filter map from the coded header.
    pub(crate) fn new(segment_number: u16) -> Self {
        Self {
            segment_number,
            elevation_segments: Vec::new(),
        }
    }
}
