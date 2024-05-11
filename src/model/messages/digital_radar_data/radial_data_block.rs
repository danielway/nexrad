use uom::si::f64::{Information, Length, Velocity};
use crate::model::messages::primitive_aliases::{Integer2, Real4, ScaledInteger2};

/// A radial data moment block.
pub struct RadialDataBlock {
    /// Data block type, "R".
    data_block_type: u8,
    
    /// Data block name, e.g. "RAD".
    data_block_name: [u8; 3],
    
    /// Size of data block in bytes.
    lrtup: Integer2,
    
    /// Unambiguous range, interval size, in km.
    unambiguous_range: ScaledInteger2,
    
    /// Noise level for the horizontal channel in dBm.
    horizontal_channel_noise_level: Real4,
    
    /// Noise level for the vertical channel in dBm.
    vertical_channel_noise_level: Real4,
    
    /// Nyquist velocity in m/s.
    nyquist_velocity: ScaledInteger2,
    
    /// Radial flags to support RPG processing.
    radial_flags: Integer2,
    
    /// Calibration constant for the horizontal channel in dBZ.
    horizontal_channel_calibration_constant: Real4,
    
    /// Calibration constant for the vertical channel in dBZ.
    vertical_channel_calibration_constant: Real4,
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
    pub fn lrtup(&self) -> Information {
        Information::new::<uom::si::information::byte>(self.lrtup as f64)
    }
    
    /// Unambiguous range, interval size.
    pub fn unambiguous_range(&self) -> Length {
        Length::new::<uom::si::length::kilometer>(self.unambiguous_range as f64)
    }
    
    /// Nyquist velocity.
    pub fn nyquist_velocity(&self) -> Velocity {
        Velocity::new::<uom::si::velocity::meter_per_second>(self.nyquist_velocity as f64)
    }
}