use crate::messages::primitive_aliases::{Code2, Integer2};
use std::fmt::Debug;
use zerocopy::{TryFromBytes, Immutable, KnownLayout};

use crate::messages::clutter_filter_map::OpCode;
#[cfg(feature = "uom")]
use uom::si::f64::Length;
#[cfg(feature = "uom")]
use uom::si::length::kilometer;

/// Defines a range segment of a particular elevation and azimuth with an operation type describing
/// the clutter filter map behavior for the segment.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash, Debug, TryFromBytes, Immutable, KnownLayout)]
pub struct RangeZone {
    /// Operation code for the range zone.
    pub op_code: Code2,

    /// Stop range per zone in km. There are 20 possible zones and not all need to be defined. The
    /// last zone must have an end range of 511km.
    pub end_range: Integer2,
}

impl RangeZone {
    /// Decodes a reference to a RangeZone from a byte slice, returning the range zone and remaining bytes.
    pub fn decode_ref(bytes: &[u8]) -> crate::result::Result<(&Self, &[u8])> {
        Ok(Self::try_ref_from_prefix(bytes)?)
    }

    /// Decodes an owned copy of a RangeZone from a byte slice.
    pub fn decode_owned(bytes: &[u8]) -> crate::result::Result<Self> {
        let (range_zone, _) = Self::decode_ref(bytes)?;
        Ok(range_zone.clone())
    }

    /// Operation code for the range zone.
    pub fn op_code(&self) -> OpCode {
        match self.op_code.get() {
            0 => OpCode::BypassFilter,
            1 => OpCode::BypassMapInControl,
            2 => OpCode::ForceFilter,
            value => panic!("Invalid OpCode: {}", value),
        }
    }

    /// Stop range per zone. There are 20 possible zones and not all need to be defined. The last
    /// zone must have an end range of 511km.
    #[cfg(feature = "uom")]
    pub fn end_range(&self) -> Length {
        Length::new::<kilometer>(self.end_range.get() as f64)
    }
}
