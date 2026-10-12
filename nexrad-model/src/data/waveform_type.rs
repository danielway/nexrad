use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Radar waveform type used for an elevation cut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum WaveformType {
    /// Contiguous Surveillance.
    ContiguousSurveillance,
    /// Contiguous Doppler with Ambiguity Resolution.
    ContiguousDopplerWithAmbiguityResolution,
    /// Contiguous Doppler without Ambiguity Resolution.
    ContiguousDopplerWithoutAmbiguityResolution,
    /// Batch.
    Batch,
    /// Staggered Pulse Pair.
    StaggeredPulsePair,
    /// Unknown waveform type.
    Unknown,
}

impl Display for WaveformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WaveformType::ContiguousSurveillance => write!(f, "Contiguous Surveillance"),
            WaveformType::ContiguousDopplerWithAmbiguityResolution => {
                write!(f, "Contiguous Doppler w/ Ambiguity Resolution")
            }
            WaveformType::ContiguousDopplerWithoutAmbiguityResolution => {
                write!(f, "Contiguous Doppler w/o Ambiguity Resolution")
            }
            WaveformType::Batch => write!(f, "Batch"),
            WaveformType::StaggeredPulsePair => write!(f, "Staggered Pulse Pair"),
            WaveformType::Unknown => write!(f, "Unknown"),
        }
    }
}
