use crate::model::messages::clutter_filter_map::elevation_segment::ElevationSegment;
use crate::model::messages::clutter_filter_map::header::Header;
use std::fmt::Debug;

/// A clutter filter map describing elevations, azimuths, and ranges containing clutter to
/// filtered from radar products.
#[derive(Debug)]
pub struct Message {
    /// Decoded header information for this clutter filter map.
    pub header: Header,

    /// The elevation segments defined in this clutter filter map.
    pub elevation_segments: Vec<ElevationSegment>,
}

impl Message {
    /// Creates a new clutter filter map from the coded header.
    pub(crate) fn new(header: Header) -> Self {
        Self {
            elevation_segments: Vec::with_capacity(header.elevation_segment_count as usize),
            header,
        }
    }
}
