use crate::messages::digital_radar_data::DataBlockId;
use crate::messages::primitive_aliases::{Integer2, Real4, ScaledSInteger2};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[cfg(feature = "uom")]
use uom::si::f64::Information;
#[cfg(feature = "uom")]
use uom::si::information::byte;

/// An elevation data block.
#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
pub struct ElevationDataBlock {
    /// Data block identifier.
    pub data_block_id: DataBlockId,

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
