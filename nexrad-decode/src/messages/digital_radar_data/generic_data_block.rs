use crate::messages::digital_radar_data::{ControlFlags, DataBlockId, ScaledMomentValue};
use crate::messages::primitive_aliases::{
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
#[derive(Clone, PartialEq)]
pub struct GenericDataBlock {
    /// The generic data block's header information.
    pub header: GenericDataBlockHeader,

    /// The generic data block's encoded moment data.
    pub encoded_data: Vec<u8>,
}

impl GenericDataBlock {
    /// Creates a new generic data moment block from the decoded header.
    pub(crate) fn new(header: GenericDataBlockHeader) -> Self {
        let word_size_bytes = header.data_word_size as usize / 8;
        let encoded_data_size = header.number_of_data_moment_gates as usize * word_size_bytes;
        Self {
            encoded_data: vec![0; encoded_data_size],
            header,
        }
    }

    /// Raw gate values for this moment/radial ordered in ascending distance from the radar. These
    /// values are stored in a fixed-point representation using the `DataMomentHeader.offset` and
    /// `DataMomentHeader.scale` fields. `decoded_data` provides decoded floating-point values.  
    pub fn encoded_values(&self) -> &[u8] {
        &self.encoded_data
    }

    /// Decodes raw moment values from `encoded_data` from their fixed-point representation into
    /// their floating point representation. Additionally, identifies special values such as "below
    /// threshold" and "range folded".
    pub fn decoded_values(&self) -> Vec<ScaledMomentValue> {
        self.encoded_data
            .iter()
            .copied()
            .map(|raw_value| {
                if self.header.scale == 0.0 {
                    return ScaledMomentValue::Value(raw_value as f32);
                }

                match raw_value {
                    0 => ScaledMomentValue::BelowThreshold,
                    1 => ScaledMomentValue::RangeFolded,
                    _ => ScaledMomentValue::Value(
                        (raw_value as f32 - self.header.offset) / self.header.scale,
                    ),
                }
            })
            .collect()
    }

    /// Get moment data from this generic data block. Note that this will clone the underlying data.
    #[cfg(feature = "nexrad-model")]
    pub fn moment_data(&self) -> nexrad_model::data::MomentData {
        nexrad_model::data::MomentData::from_fixed_point(
            self.header.scale,
            self.header.offset,
            self.encoded_data.clone(),
        )
    }

    /// Convert this generic data block into common model moment data, minimizing data copies.
    #[cfg(feature = "nexrad-model")]
    pub fn into_moment_data(self) -> nexrad_model::data::MomentData {
        nexrad_model::data::MomentData::from_fixed_point(
            self.header.scale,
            self.header.offset,
            self.encoded_data,
        )
    }
}

impl Debug for GenericDataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericDataBlock")
            .field("header", &self.header)
            .field("data", &self.encoded_data.len())
            .finish()
    }
}

/// A generic data moment block's decoded header.
#[derive(Clone, PartialEq, Deserialize)]
pub struct GenericDataBlockHeader {
    /// Data block identifier.
    pub data_block_id: DataBlockId,

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
            .field("data_block_id", &self.data_block_id)
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
            .field("data_block_id", &self.data_block_id)
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
