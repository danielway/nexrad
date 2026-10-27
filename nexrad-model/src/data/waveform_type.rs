use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Radar waveform type used for an elevation cut.
///
/// `nexrad_decode::messages::volume_coverage_pattern::WaveformType` has the same variants and
/// converts to and from this type without loss; it also maps each variant to its ICD code.
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
    /// A waveform type the ICD does not define, carrying the code the decoder read.
    Unknown(u8),
}

impl WaveformType {
    /// The abbreviation the RDA/RPG ICD's volume coverage pattern tables use for this waveform
    /// type: `CS`, `CD/W`, `CD/WO` or `B`.
    ///
    /// Returns `None` for [`StaggeredPulsePair`](Self::StaggeredPulsePair), which the ICD's
    /// waveform type legend does not abbreviate, and for [`Unknown`](Self::Unknown).
    pub fn abbreviation(self) -> Option<&'static str> {
        match self {
            WaveformType::ContiguousSurveillance => Some("CS"),
            WaveformType::ContiguousDopplerWithAmbiguityResolution => Some("CD/W"),
            WaveformType::ContiguousDopplerWithoutAmbiguityResolution => Some("CD/WO"),
            WaveformType::Batch => Some("B"),
            WaveformType::StaggeredPulsePair | WaveformType::Unknown(_) => None,
        }
    }
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
            WaveformType::Unknown(code) => write!(f, "Unknown ({code})"),
        }
    }
}
