use uom::si::f64::Length;
use uom::si::length::kilometer;
use crate::model::messages::digital_radar_data::ControlFlags;
use crate::model::messages::primitive_aliases::{Code1, Integer1, Integer2, Integer4, Real4, ScaledInteger2};

/// A generic data moment block.
pub struct GenericDataBlock {
    /// Data moment type, "D".
    data_block_type: u8,
    
    /// Data moment name, e.g. "VEL", "REF", "SW".
    data_block_name: [u8; 3],
    
    /// Reserved.
    reserved: Integer4,
    
    /// Number of data moment gates for current radial, from 0 to 1840.
    number_of_data_moment_gates: Integer2,
    
    /// Range to center of first range gate in 0.000-scaled kilometers.
    data_moment_range: ScaledInteger2,
    
    /// Size of data moment sample interval in 0.00-scaled kilometers from 0.25 to 4.0.
    data_moment_range_sample_interval: ScaledInteger2,
    
    /// Threshold parameter specifying the minimum difference in echo power between two resolution
    /// gates in dB for them to not be labeled as "overlayed".
    tover: ScaledInteger2,
    
    /// Signal-to-noise ratio threshold for valid data from -12 to 20 dB.
    snr_threshold: ScaledInteger2,
    
    /// Flags indicating special control features.
    /// 
    /// Flags:
    ///   0 = None
    ///   1 = Recombined azimuthal radials
    ///   2 = Recombined range gates
    ///   3 = Recombined radials and range gates to legacy resolution
    control_flags: Code1,
    
    /// Number of bits (8 or 16) used for storing data for each data moment gate.
    data_word_size: Integer1,
    
    /// Scale factor for converting data moments to floating-point representation.
    scale: Real4,
    
    /// Offset value for converting data moments to floating-point representation.
    offset: Real4,
}

impl GenericDataBlock {
    /// Data moment type, "D".
    pub fn data_block_type(&self) -> char {
        self.data_block_type as char
    }

    /// Data moment name, e.g. "VEL", "REF", "SW".
    pub fn data_block_name(&self) -> String {
        String::from_utf8_lossy(&self.data_block_name).to_string()
    }
    
    /// Range to center of first range gate.
    pub fn data_moment_range(&self) -> Length {
        Length::new::<kilometer>(self.data_moment_range as f64 * 0.001)
    }

    /// Size of data moment sample interval.
    pub fn data_moment_range_sample_interval(&self) -> Length {
        Length::new::<kilometer>(self.data_moment_range_sample_interval as f64 * 0.01)
    }
    
    /// Flags indicating special control features.
    pub fn control_flags(&self) -> ControlFlags {
        match self.control_flags {
            0 => ControlFlags::None,
            1 => ControlFlags::RecombinedAzimuthalRadials,
            2 => ControlFlags::RecombinedRangeGates,
            3 => ControlFlags::RecombinedRadialsAndRangeGatesToLegacyResolution,
            _ => panic!("Invalid control flag value: {}", self.control_flags)
        }
    }
}
