use crate::messages::clutter_filter_map::azimuth_segment::AzimuthSegment;
use crate::messages::primitive_aliases::Integer1;

/// A segment of the clutter filter map for a specific elevation containing azimuth segments.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ElevationSegment {
    /// This elevation segment's number from 1 to 5 (oftentimes there are only 2) in increasing
    /// elevation from the ground.
    pub elevation_segment_number: Integer1,

    /// The azimuth segments defined in this elevation segment.
    pub azimuth_segments: Vec<AzimuthSegment>,
}

impl ElevationSegment {
    /// Creates a new elevation segment to contain azimuth segments.
    pub(crate) fn new(elevation_segment_number: Integer1) -> Self {
        Self {
            elevation_segment_number,
            azimuth_segments: Vec::with_capacity(360),
        }
    }
}
