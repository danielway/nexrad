use crate::messages::clutter_filter_map::{AzimuthSegmentHeader, RangeZone};
use crate::result::Result;
use crate::util::take_ref;

/// A segment of the clutter filter map for a specific elevation and azimuth containing range zones.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AzimuthSegment<'a> {
    /// Header information for this azimuth segment. This is the portion of an azimuth segment that
    /// is read directly from the Archive II file.
    pub header: &'a AzimuthSegmentHeader,

    /// This azimuth segment's number from 0 to 359. Each azimuth segment subtends a range of 1
    /// degree, e.g.: 0 degrees <= azimuth segment 0 < 1 degree.
    pub azimuth_segment: u16,

    /// The range zones defined in this azimuth segment.
    pub range_zones: Vec<&'a RangeZone>,
}

impl<'a> AzimuthSegment<'a> {
    /// Parse an azimuth segment (expected to be the specified number) from the input.
    pub(crate) fn parse<'b>(input: &'b mut &'a [u8], segment_number: u16) -> Result<Self> {
        let header = take_ref::<AzimuthSegmentHeader>(input)?;

        let range_zone_count = header.range_zone_count.get() as usize;
        let mut azimuth_segment = AzimuthSegment {
            header,
            azimuth_segment: segment_number,
            range_zones: Vec::with_capacity(header.range_zone_count.get() as usize),
        };

        for _ in 0..range_zone_count {
            let range_zone = take_ref::<RangeZone>(input)?;
            azimuth_segment.range_zones.push(range_zone);
        }

        Ok(azimuth_segment)
    }
}
