use crate::messages::clutter_filter_map::OpCode;
use crate::messages::primitive_aliases::{Code2, Integer2};
use serde::Deserialize;
use std::fmt::Debug;

#[cfg(feature = "uom")]
use uom::{si::f64::Length, si::length::kilometer};

/// Defines a range segment of a particular elevation and azimuth with an operation type describing
/// the clutter filter map behavior for the segment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct RangeZone {
    /// Operation code for the range zone.
    pub op_code: Code2,

    /// Stop range per zone in km. There are 20 possible zones and not all need to be defined. The
    /// last zone must have an end range of 511km.
    pub end_range: Integer2,
}

impl RangeZone {
    /// Operation code for the range zone.
    pub fn op_code(&self) -> OpCode {
        match self.op_code {
            0 => OpCode::BypassFilter,
            1 => OpCode::BypassMapInControl,
            2 => OpCode::ForceFilter,
            _ => panic!("Invalid OpCode: {}", self.op_code),
        }
    }

    /// Stop range per zone. There are 20 possible zones and not all need to be defined. The last
    /// zone must have an end range of 511km.
    #[cfg(feature = "uom")]
    pub fn end_range(&self) -> Length {
        Length::new::<kilometer>(self.end_range as f64)
    }
}
