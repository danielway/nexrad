use crate::model::messages::digital_radar_data::ControlFlags;
use crate::model::messages::primitive_aliases::{
    Code1, Integer1, Integer2, Integer4, Real4, ScaledInteger2,
};
use serde::Deserialize;
use std::fmt::Debug;

#[cfg(feature = "uom")]
use uom::si::f64::{Information, Length};
#[cfg(feature = "uom")]
use uom::si::information::byte;
#[cfg(feature = "uom")]
use uom::si::length::kilometer;

/// A generic data moment block.
pub struct GenericDataBlock {
    /// The generic data block's header information.
    pub header: GenericDataBlockHeader,

    /// The generic data block's moment data.
    pub data: Vec<u8>,
}

impl GenericDataBlock {
    /// Creates a new generic data moment block from the decoded header.
    pub(crate) fn new(header: GenericDataBlockHeader) -> Self {
        Self {
            data: vec![
                0;
                (header.number_of_data_moment_gates * header.data_word_size as u16) as usize
            ],
            header,
        }
    }
}

impl Debug for GenericDataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericDataBlock")
            .field("header", &self.header)
            .field("data", &self.data.len())
            .finish()
    }
}

/// A generic data moment block's decoded header.
#[derive(Deserialize)]
pub struct GenericDataBlockHeader {
    /// Data moment type, "D".
    pub data_block_type: u8,

    /// Data moment name, e.g. "VEL", "REF", "SW".
    pub data_block_name: [u8; 3],

    /// Reserved.
    pub reserved: Integer4,

    /// Number of data moment gates for current radial, from 0 to 1840.
    pub number_of_data_moment_gates: Integer2,

    /// Range to center of first range gate in 0.000-scaled kilometers.
    pub data_moment_range: ScaledInteger2,

    /// Size of data moment sample interval in 0.00-scaled kilometers from 0.25 to 4.0.
    pub data_moment_range_sample_interval: ScaledInteger2,

    /// Threshold parameter specifying the minimum difference in echo power between two resolution
    /// gates in dB for them to not be labeled as "overlayed".
    pub tover: ScaledInteger2,

    /// Signal-to-noise ratio threshold for valid data from -12 to 20 dB.
    pub snr_threshold: ScaledInteger2,

    /// Flags indicating special control features.
    ///
    /// Flags:
    ///   0 = None
    ///   1 = Recombined azimuthal radials
    ///   2 = Recombined range gates
    ///   3 = Recombined radials and range gates to legacy resolution
    pub control_flags: Code1,

    /// Number of bits (8 or 16) used for storing data for each data moment gate.
    pub data_word_size: Integer1,

    /// Scale factor for converting data moments to floating-point representation.
    pub scale: Real4,

    /// Offset value for converting data moments to floating-point representation.
    pub offset: Real4,
}

impl GenericDataBlockHeader {
    /// Data moment type, "D".
    pub fn data_block_type(&self) -> char {
        self.data_block_type as char
    }

    /// Data moment name, e.g. "VEL", "REF", "SW".
    pub fn data_block_name(&self) -> String {
        String::from_utf8_lossy(&self.data_block_name).to_string()
    }

    /// Range to center of first range gate.
    #[cfg(feature = "uom")]
    pub fn data_moment_range(&self) -> Length {
        Length::new::<kilometer>(self.data_moment_range as f64 * 0.001)
    }

    /// Size of data moment sample interval.
    #[cfg(feature = "uom")]
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
            _ => panic!("Invalid control flag value: {}", self.control_flags),
        }
    }

    /// Size of the data moment block in bytes.
    #[cfg(feature = "uom")]
    pub fn moment_size(&self) -> Information {
        Information::new::<byte>(
            self.number_of_data_moment_gates as f64 * self.data_word_size as f64 / 8.0,
        )
    }
}

#[cfg(not(feature = "uom"))]
impl Debug for GenericDataBlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericDataBlockHeader")
            .field("data_block_type", &self.data_block_type())
            .field("data_block_name", &self.data_block_name())
            .field("reserved", &self.reserved)
            .field(
                "number_of_data_moment_gates",
                &self.number_of_data_moment_gates,
            )
            .field("data_moment_range", &self.data_moment_range)
            .field(
                "data_moment_range_sample_interval",
                &self.data_moment_range_sample_interval,
            )
            .field("tover", &self.tover)
            .field("snr_threshold", &self.snr_threshold)
            .field("control_flags", &self.control_flags())
            .field("data_word_size", &self.data_word_size)
            .field("scale", &self.scale)
            .field("offset", &self.offset)
            .finish()
    }
}

#[cfg(feature = "uom")]
impl Debug for GenericDataBlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericDataBlockHeader")
            .field("data_block_type", &self.data_block_type())
            .field("data_block_name", &self.data_block_name())
            .field("reserved", &self.reserved)
            .field(
                "number_of_data_moment_gates",
                &self.number_of_data_moment_gates,
            )
            .field("data_moment_range", &self.data_moment_range())
            .field(
                "data_moment_range_sample_interval",
                &self.data_moment_range_sample_interval(),
            )
            .field("tover", &self.tover)
            .field("snr_threshold", &self.snr_threshold)
            .field("control_flags", &self.control_flags())
            .field("data_word_size", &self.data_word_size)
            .field("scale", &self.scale)
            .field("offset", &self.offset)
            .finish()
    }
}
