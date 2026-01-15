use crate::messages::primitive_aliases::{
    Code1, Integer1, Integer2, Integer4, Real4, ScaledInteger2,
};
use std::fmt::Debug;
use zerocopy::{FromBytes, Immutable, KnownLayout};

/// A generic data moment block's decoded header.
#[repr(C)]
#[derive(Clone, PartialEq, Debug, FromBytes, Immutable, KnownLayout)]
pub struct GenericDataBlockHeader {
    /// Reserved.
    pub(crate) reserved: Integer4,

    /// Number of data moment gates for current radial, from 0 to 1840.
    pub(crate) number_of_data_moment_gates: Integer2,

    /// Range to center of first range gate in 0.000-scaled kilometers.
    pub(crate) data_moment_range: ScaledInteger2,

    /// Size of data moment sample interval in 0.000-scaled kilometers from 0.25 to 4.0.
    pub(crate) data_moment_range_sample_interval: ScaledInteger2,

    /// Threshold parameter specifying the minimum difference in echo power between two resolution
    /// gates in dB for them to not be labeled as "overlayed".
    pub(crate) tover: ScaledInteger2,

    /// Signal-to-noise ratio threshold for valid data from -12 to 20 dB.
    pub(crate) snr_threshold: ScaledInteger2,

    /// Flags indicating special control features.
    ///
    /// Flags:
    ///   0 = None
    ///   1 = Recombined azimuthal radials
    ///   2 = Recombined range gates
    ///   3 = Recombined radials and range gates to legacy resolution
    pub(crate) control_flags: Code1,

    /// Number of bits (8 or 16) used for storing data for each data moment gate.
    pub(crate) data_word_size: Integer1,

    /// Scale factor for converting data moments to floating-point representation.
    pub(crate) scale: Real4,

    /// Offset value for converting data moments to floating-point representation.
    pub(crate) offset: Real4,
}
