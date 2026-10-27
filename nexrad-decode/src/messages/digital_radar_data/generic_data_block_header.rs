use super::raw;
use super::{ControlFlags, DataWordSize};
use std::borrow::Cow;


/// A generic data moment block's decoded header.
#[derive(Clone, PartialEq, Debug)]
pub struct GenericDataBlockHeader<'a> {
    inner: Cow<'a, raw::GenericDataBlockHeader>,
}

impl<'a> GenericDataBlockHeader<'a> {
    /// Create a new GenericDataBlockHeader wrapper from a raw GenericDataBlockHeader reference.
    pub(crate) fn new(inner: &'a raw::GenericDataBlockHeader) -> Self {
        Self {
            inner: Cow::Borrowed(inner),
        }
    }

    /// Convert this generic data block header to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> GenericDataBlockHeader<'static> {
        GenericDataBlockHeader {
            inner: Cow::Owned(self.inner.into_owned()),
        }
    }

    /// Number of data moment gates for current radial, from 0 to 1840.
    pub fn data_moment_gate_count(&self) -> u16 {
        self.inner.data_moment_gate_count.get()
    }

    /// Range to center of first range gate in 0.000-scaled kilometers (raw value).
    pub fn data_moment_range_raw(&self) -> u16 {
        self.inner.data_moment_range.get()
    }

    /// Size of data moment sample interval in 0.000-scaled kilometers (raw value).
    pub fn data_moment_range_sample_interval_raw(&self) -> u16 {
        self.inner.data_moment_range_sample_interval.get()
    }

    /// Threshold parameter specifying the minimum difference in echo power between two resolution
    /// gates for them to not be labeled as "overlayed", in tenths of a dB (raw scaled value). See
    /// [`tover_db()`](Self::tover_db) for the decoded value.
    pub fn tover_raw(&self) -> u16 {
        self.inner.tover.get()
    }

    /// Signal-to-noise ratio threshold for valid data, in eighths of a dB (raw scaled value). See
    /// [`snr_threshold_db()`](Self::snr_threshold_db) for the decoded value.
    pub fn snr_threshold_raw(&self) -> i16 {
        self.inner.snr_threshold.get()
    }

    /// Flags indicating special control features (raw value).
    pub fn control_flags_raw(&self) -> u8 {
        self.inner.control_flags
    }

    /// Number of bits (8 or 16) used for storing data for each data moment gate (raw value).
    pub fn data_word_size_raw(&self) -> u8 {
        self.inner.data_word_size
    }

    /// Scale factor for converting data moments to floating-point representation.
    pub fn scale(&self) -> f32 {
        self.inner.scale.get()
    }

    /// Offset value for converting data moments to floating-point representation.
    pub fn offset(&self) -> f32 {
        self.inner.offset.get()
    }

    /// Threshold parameter specifying the minimum difference in echo power between two resolution
    /// gates for them to not be labeled as "overlayed", in dB.
    pub fn tover_db(&self) -> f32 {
        self.inner.tover.get() as f32 * 0.1
    }

    /// Signal-to-noise ratio threshold for valid data from -12 to 20 dB, in dB.
    pub fn snr_threshold_db(&self) -> f32 {
        self.inner.snr_threshold.get() as f32 * 0.125
    }

    /// Range to center of first range gate in kilometers.
    pub fn data_moment_range_km(&self) -> f64 {
        self.inner.data_moment_range.get() as f64 * 0.001
    }

    /// Size of data moment sample interval in kilometers.
    pub fn data_moment_range_sample_interval_km(&self) -> f64 {
        self.inner.data_moment_range_sample_interval.get() as f64 * 0.001
    }

    /// Number of bits used for storing data for each data moment gate.
    pub fn data_word_size(&self) -> DataWordSize {
        match self.inner.data_word_size {
            8 => DataWordSize::EightBit,
            16 => DataWordSize::SixteenBit,
            other => DataWordSize::Unknown(other),
        }
    }

    /// Flags indicating special control features.
    pub fn control_flags(&self) -> ControlFlags {
        match self.inner.control_flags {
            0 => ControlFlags::None,
            1 => ControlFlags::RecombinedAzimuthalRadials,
            2 => ControlFlags::RecombinedRangeGates,
            3 => ControlFlags::RecombinedRadialsAndRangeGatesToLegacyResolution,
            other => ControlFlags::Unknown(other),
        }
    }

    /// Size of the data moment block, in bytes (gate count times word size in bits, divided by
    /// 8).
    pub fn moment_size_bytes(&self) -> u32 {
        self.inner.data_moment_gate_count.get() as u32 * self.inner.data_word_size as u32 / 8
    }

}
