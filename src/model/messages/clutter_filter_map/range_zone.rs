use crate::model::messages::primitive_aliases::{Code2, Integer2};
use serde::Deserialize;
use std::fmt::Debug;
use uom::si::f64::Length;
use uom::si::length::kilometer;

/// Defines a range segment of a particular elevation and azimuth with an operation type describing
/// the clutter filter map behavior for the segment.
#[derive(Deserialize)]
pub struct RangeZone {
    /// Bypass filter, bypass map in control force filter. E.g. 0, 1, or 2.
    // todo: define numeration of codes with meaning
    pub op_code: Code2,

    /// Stop range per zone in km. There are 20 possible zones and not all need to be defined. The
    /// last zone must have an end range of 511km.
    pub end_range: Integer2,
}

impl RangeZone {
    /// Stop range per zone. There are 20 possible zones and not all need to be defined. The last
    /// zone must have an end range of 511km.
    pub fn end_range(&self) -> Length {
        Length::new::<kilometer>(self.end_range as f64)
    }
}

impl Debug for RangeZone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RangeZone")
            .field("op_code", &self.op_code)
            .field("end_range", &self.end_range())
            .finish()
    }
}
