use crate::model::messages::clutter_filter_map::range_zone::RangeZone;
use crate::model::messages::primitive_aliases::Integer2;
use serde::Deserialize;

/// Header information for an azimuth segment to be read directly from the Archive II file.
#[derive(Debug, Deserialize)]
pub struct AzimuthSegmentHeader {
    /// The number of range zones defined in this azimuth segment, from 1 to 20.
    pub range_zone_count: Integer2,
}

/// A segment of the clutter filter map for a specific elevation and azimuth containing range zones.
#[derive(Debug)]
pub struct AzimuthSegment {
    /// Header information for this azimuth segment. This is the portion of an azimuth segment that
    /// is read directly from the Archive II file.
    pub header: AzimuthSegmentHeader,

    /// This azimuth segment's number from 0 to 359. Each azimuth segment subtends a range of 1
    /// degree, e.g.: 0 degrees <= azimuth segment 0 < 1 degree.
    pub azimuth_segment: Integer2,

    /// The range zones defined in this azimuth segment.
    pub range_zones: Vec<RangeZone>,
}

impl AzimuthSegment {
    /// Creates a new azimuth segment from the coded header.
    pub(crate) fn new(header: AzimuthSegmentHeader, azimuth_segment: Integer2) -> Self {
        Self {
            range_zones: Vec::with_capacity(header.range_zone_count as usize),
            header,
            azimuth_segment,
        }
    }
}
