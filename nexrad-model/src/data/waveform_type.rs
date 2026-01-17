use std::fmt::{Debug, Display};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Radar waveform type used for an elevation cut.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum WaveformType {
    /// Contiguous Surveillance.
    CS,
    /// Contiguous Doppler with Ambiguity Resolution.
    CDW,
    /// Contiguous Doppler without Ambiguity Resolution.
    CDWO,
    /// Batch.
    B,
    /// Staggered Pulse Pair.
    SPP,
    /// Unknown waveform type.
    Unknown,
}

impl Display for WaveformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WaveformType::CS => write!(f, "Contiguous Surveillance"),
            WaveformType::CDW => write!(f, "Contiguous Doppler w/ Ambiguity Resolution"),
            WaveformType::CDWO => write!(f, "Contiguous Doppler w/o Ambiguity Resolution"),
            WaveformType::B => write!(f, "Batch"),
            WaveformType::SPP => write!(f, "Staggered Pulse Pair"),
            WaveformType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Debug for WaveformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WaveformType::CS => write!(f, "CS"),
            WaveformType::CDW => write!(f, "CDW"),
            WaveformType::CDWO => write!(f, "CDWO"),
            WaveformType::B => write!(f, "B"),
            WaveformType::SPP => write!(f, "SPP"),
            WaveformType::Unknown => write!(f, "Unknown"),
        }
    }
}
