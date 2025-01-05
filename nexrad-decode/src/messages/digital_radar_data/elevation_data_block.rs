use crate::messages::primitive_aliases::{Integer2, Real4, ScaledSInteger2};
use serde::Deserialize;
use std::fmt::Debug;

#[cfg(feature = "uom")]
use uom::si::f64::Information;
#[cfg(feature = "uom")]
use uom::si::information::byte;

/// An elevation data block.
#[derive(Clone, PartialEq, Deserialize)]
pub struct ElevationDataBlock {
    /// Size of data block in bytes.
    pub lrtup: Integer2,

    /// Atmospheric attenuation factor in dB/km.
    pub atmos: ScaledSInteger2,

    /// Scaling constant used by the signal processor for this elevation to calculate reflectivity
    /// in dB.
    pub calibration_constant: Real4,
}

impl ElevationDataBlock {
    /// Size of data block.
    #[cfg(feature = "uom")]
    pub fn lrtup(&self) -> Information {
        Information::new::<byte>(self.lrtup as f64)
    }
}

#[cfg(not(feature = "uom"))]
impl Debug for ElevationDataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElevationDataBlock")
            .field("lrtup", &self.lrtup)
            .field("atmos", &self.atmos)
            .field("calibration_constant", &self.calibration_constant)
            .finish()
    }
}

#[cfg(feature = "uom")]
impl Debug for ElevationDataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElevationDataBlock")
            .field("lrtup", &self.lrtup())
            .field("atmos", &self.atmos)
            .field("calibration_constant", &self.calibration_constant)
            .finish()
    }
}
