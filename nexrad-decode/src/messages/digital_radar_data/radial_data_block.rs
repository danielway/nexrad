use crate::messages::digital_radar_data::DataBlockId;
use crate::messages::primitive_aliases::{Integer2, Real4, ScaledInteger2};
use std::fmt::Debug;
use zerocopy::{TryFromBytes, Immutable, KnownLayout};

#[cfg(feature = "uom")]
use uom::si::f64::{Information, Length, Velocity};

/// A radial data moment block.
#[repr(C)]
#[derive(Clone, PartialEq, Debug, TryFromBytes, Immutable, KnownLayout)]
pub struct RadialDataBlock {
    /// Data block identifier.
    pub data_block_id: DataBlockId,

    /// Size of data block in bytes.
    pub lrtup: Integer2,

    /// Unambiguous range, interval size, in km.
    pub unambiguous_range: ScaledInteger2,

    /// Noise level for the horizontal channel in dBm.
    pub horizontal_channel_noise_level: Real4,

    /// Noise level for the vertical channel in dBm.
    pub vertical_channel_noise_level: Real4,

    /// Nyquist velocity in m/s.
    pub nyquist_velocity: ScaledInteger2,

    /// Radial flags to support RPG processing.
    pub radial_flags: Integer2,

    /// Calibration constant for the horizontal channel in dBZ.
    pub horizontal_channel_calibration_constant: Real4,

    /// Calibration constant for the vertical channel in dBZ.
    pub vertical_channel_calibration_constant: Real4,
}

impl RadialDataBlock {
    /// Size of data block.
    #[cfg(feature = "uom")]
    pub fn lrtup(&self) -> Information {
        Information::new::<uom::si::information::byte>(self.lrtup.get() as f64)
    }

    /// Unambiguous range, interval size.
    #[cfg(feature = "uom")]
    pub fn unambiguous_range(&self) -> Length {
        Length::new::<uom::si::length::kilometer>(self.unambiguous_range.get() as f64)
    }

    /// Nyquist velocity.
    #[cfg(feature = "uom")]
    pub fn nyquist_velocity(&self) -> Velocity {
        Velocity::new::<uom::si::velocity::meter_per_second>(self.nyquist_velocity.get() as f64 * 0.01)
    }

    /// Decodes a reference to a RadialDataBlock from a byte slice, returning the block and remaining bytes.
    pub fn decode_ref(bytes: &[u8]) -> crate::result::Result<(&Self, &[u8])> {
        Ok(Self::try_ref_from_prefix(bytes)?)
    }

    /// Decodes an owned copy of a RadialDataBlock from a byte slice.
    pub fn decode_owned(bytes: &[u8]) -> crate::result::Result<Self> {
        let (block, _) = Self::decode_ref(bytes)?;
        Ok(block.clone())
    }
}
