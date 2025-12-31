use crate::messages::clutter_filter_map::{AzimuthSegmentHeader, RangeZone};
use crate::result::Result;
use crate::slice_reader::SliceReader;
use std::borrow::Cow;

/// A segment of the clutter filter map for a specific elevation and azimuth containing range zones.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AzimuthSegment<'a> {
    /// Header information for this azimuth segment. This is the portion of an azimuth segment that
    /// is read directly from the Archive II file.
    pub header: Cow<'a, AzimuthSegmentHeader>,

    /// This azimuth segment's number from 0 to 359. Each azimuth segment subtends a range of 1
    /// degree, e.g.: 0 degrees <= azimuth segment 0 < 1 degree.
    pub azimuth_segment: u16,

    /// The range zones defined in this azimuth segment.
    pub range_zones: Cow<'a, [RangeZone]>,
}

impl<'a> AzimuthSegment<'a> {
    /// Parse an azimuth segment (expected to be the specified number) from the reader.
    pub(crate) fn parse(reader: &mut SliceReader<'a>, segment_number: u16) -> Result<Self> {
        let header = reader.take_ref::<AzimuthSegmentHeader>()?;
        let range_zones = reader.take_slice::<RangeZone>(header.range_zone_count.get() as usize)?;

        Ok(AzimuthSegment {
            header: Cow::Borrowed(header),
            azimuth_segment: segment_number,
            range_zones: Cow::Borrowed(range_zones),
        })
    }

    /// Convert this segment to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> AzimuthSegment<'static> {
        AzimuthSegment {
            header: Cow::Owned(self.header.into_owned()),
            azimuth_segment: self.azimuth_segment,
            range_zones: Cow::Owned(self.range_zones.into_owned()),
        }
    }
}
