use crate::messages::digital_radar_data::DataBlockId;
use crate::messages::primitive_aliases::{Integer2, Real4, ScaledSInteger2};
use std::fmt::Debug;
use zerocopy::{TryFromBytes, Immutable, KnownLayout};

#[cfg(feature = "uom")]
use uom::si::f64::Information;
#[cfg(feature = "uom")]
use uom::si::information::byte;

/// An elevation data block.
#[repr(C)]
#[derive(Clone, PartialEq, Debug, TryFromBytes, Immutable, KnownLayout)]
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
        Information::new::<byte>(self.lrtup.get() as f64)
    }

    /// Decodes a reference to an ElevationDataBlock from a byte slice, returning the block and remaining bytes.
    pub fn decode_ref(bytes: &[u8]) -> crate::result::Result<(&Self, &[u8])> {
        Ok(Self::try_ref_from_prefix(bytes)?)
    }

    /// Decodes an owned copy of an ElevationDataBlock from a byte slice.
    pub fn decode_owned(bytes: &[u8]) -> crate::result::Result<Self> {
        let (block, _) = Self::decode_ref(bytes)?;
        Ok(block.clone())
    }
}
