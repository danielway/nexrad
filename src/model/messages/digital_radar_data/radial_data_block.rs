use crate::model::messages::primitive_aliases::{Integer2, Real4, ScaledInteger2};
use serde::Deserialize;
use std::fmt::Debug;

#[cfg(feature = "uom")]
use uom::si::f64::{Information, Length, Velocity};

/// A radial data moment block.
#[derive(Deserialize)]
pub struct RadialDataBlock {
    /// Data block type, "R".
    pub data_block_type: u8,

    /// Data block name, e.g. "RAD".
    pub data_block_name: [u8; 3],

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
    /// Data block type, "R".
    pub fn data_block_type(&self) -> char {
        self.data_block_type as char
    }

    /// Data block name, e.g. "RAD".
    pub fn data_block_name(&self) -> String {
        String::from_utf8_lossy(&self.data_block_name).to_string()
    }

    /// Size of data block.
    #[cfg(feature = "uom")]
    pub fn lrtup(&self) -> Information {
        Information::new::<uom::si::information::byte>(self.lrtup as f64)
    }

    /// Unambiguous range, interval size.
    #[cfg(feature = "uom")]
    pub fn unambiguous_range(&self) -> Length {
        Length::new::<uom::si::length::kilometer>(self.unambiguous_range as f64)
    }

    /// Nyquist velocity.
    #[cfg(feature = "uom")]
    pub fn nyquist_velocity(&self) -> Velocity {
        Velocity::new::<uom::si::velocity::meter_per_second>(self.nyquist_velocity as f64)
    }
}

#[cfg(not(feature = "uom"))]
impl Debug for RadialDataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RadialDataBlock")
            .field("data_block_type", &self.data_block_type())
            .field("data_block_name", &self.data_block_name())
            .field("lrtup", &self.lrtup)
            .field("unambiguous_range", &self.unambiguous_range)
            .field(
                "horizontal_channel_noise_level",
                &self.horizontal_channel_noise_level,
            )
            .field(
                "vertical_channel_noise_level",
                &self.vertical_channel_noise_level,
            )
            .field("nyquist_velocity", &self.nyquist_velocity)
            .field("radial_flags", &self.radial_flags)
            .field(
                "horizontal_channel_calibration_constant",
                &self.horizontal_channel_calibration_constant,
            )
            .field(
                "vertical_channel_calibration_constant",
                &self.vertical_channel_calibration_constant,
            )
            .finish()
    }
}

#[cfg(feature = "uom")]
impl Debug for RadialDataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RadialDataBlock")
            .field("data_block_type", &self.data_block_type())
            .field("data_block_name", &self.data_block_name())
            .field("lrtup", &self.lrtup())
            .field("unambiguous_range", &self.unambiguous_range())
            .field(
                "horizontal_channel_noise_level",
                &self.horizontal_channel_noise_level,
            )
            .field(
                "vertical_channel_noise_level",
                &self.vertical_channel_noise_level,
            )
            .field("nyquist_velocity", &self.nyquist_velocity())
            .field("radial_flags", &self.radial_flags)
            .field(
                "horizontal_channel_calibration_constant",
                &self.horizontal_channel_calibration_constant,
            )
            .field(
                "vertical_channel_calibration_constant",
                &self.vertical_channel_calibration_constant,
            )
            .finish()
    }
}
