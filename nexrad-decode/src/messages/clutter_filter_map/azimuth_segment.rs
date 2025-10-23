use crate::messages::clutter_filter_map::range_zone::RangeZone;
use crate::messages::primitive_aliases::Integer2;
use zerocopy::{TryFromBytes, Immutable, KnownLayout};

/// Header information for an azimuth segment to be read directly from the Archive II file.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, TryFromBytes, Immutable, KnownLayout)]
pub struct AzimuthSegmentHeader {
    /// The number of range zones defined in this azimuth segment, from 1 to 20.
    pub range_zone_count: Integer2,
}

impl AzimuthSegmentHeader {
    /// Decodes a reference to a AzimuthSegmentHeader from a byte slice, returning the header and remaining bytes.
    pub fn decode_ref(bytes: &[u8]) -> crate::result::Result<(&Self, &[u8])> {
        Ok(Self::try_ref_from_prefix(bytes)?)
    }

    /// Decodes an owned copy of a AzimuthSegmentHeader from a byte slice.
    pub fn decode_owned(bytes: &[u8]) -> crate::result::Result<Self> {
        let (header, _) = Self::decode_ref(bytes)?;
        Ok(header.clone())
    }
}

/// A segment of the clutter filter map for a specific elevation and azimuth containing range zones.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AzimuthSegment {
    /// Header information for this azimuth segment. This is the portion of an azimuth segment that
    /// is read directly from the Archive II file.
    pub header: AzimuthSegmentHeader,

    /// This azimuth segment's number from 0 to 359. Each azimuth segment subtends a range of 1
    /// degree, e.g.: 0 degrees <= azimuth segment 0 < 1 degree.
    pub azimuth_segment: u16,

    /// The range zones defined in this azimuth segment.
    pub range_zones: Vec<RangeZone>,
}

impl AzimuthSegment {
    /// Creates a new azimuth segment from the coded header.
    pub(crate) fn new(header: AzimuthSegmentHeader, azimuth_segment: u16) -> Self {
        Self {
            range_zones: Vec::with_capacity(header.range_zone_count.get() as usize),
            header,
            azimuth_segment,
        }
    }
}
