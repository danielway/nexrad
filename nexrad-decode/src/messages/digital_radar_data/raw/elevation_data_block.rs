use crate::messages::primitive_aliases::{Integer2, Real4, ScaledSInteger2};
use std::fmt::Debug;
use zerocopy::{FromBytes, Immutable, KnownLayout};

/// An elevation data block.
#[repr(C)]
#[derive(Clone, PartialEq, Debug, FromBytes, Immutable, KnownLayout)]
pub struct ElevationDataBlock {
    /// Size of data block in bytes.
    pub(crate) lrtup: Integer2,

    /// Atmospheric attenuation factor in dB/km.
    pub(crate) atmos: ScaledSInteger2,

    /// Scaling constant used by the signal processor for this elevation to calculate reflectivity
    /// in dB.
    pub(crate) calibration_constant: Real4,
}
