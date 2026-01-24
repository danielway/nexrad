use crate::messages::primitive_aliases::{Integer2, Real4, ScaledInteger2};
use std::fmt::Debug;
use zerocopy::{FromBytes, Immutable, KnownLayout};

/// A radial data moment block.
#[repr(C)]
#[derive(Clone, PartialEq, Debug, FromBytes, Immutable, KnownLayout)]
pub struct RadialDataBlock {
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
