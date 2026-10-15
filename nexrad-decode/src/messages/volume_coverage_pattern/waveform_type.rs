/// Possible values for waveform type.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum WaveformType {
    /// Contiguous Surveillance.
    ContiguousSurveillance,
    /// Contiguous Doppler with Ambiguity Resolution.
    ContiguousDopplerWithAmbiguityResolution,
    /// Contiguous Doppler without Ambiguity Resolution.
    ContiguousDopplerWithoutAmbiguityResolution,
    /// Batch mode.
    Batch,
    /// Staggered Pulse Pair.
    StaggeredPulsePair,
    /// Unknown waveform type.
    Unknown,
}

impl WaveformType {
    /// Decode a waveform type code.
    pub(crate) fn from_raw(code: u8) -> Self {
        match code {
            1 => WaveformType::ContiguousSurveillance,
            2 => WaveformType::ContiguousDopplerWithAmbiguityResolution,
            3 => WaveformType::ContiguousDopplerWithoutAmbiguityResolution,
            4 => WaveformType::Batch,
            5 => WaveformType::StaggeredPulsePair,
            _ => WaveformType::Unknown,
        }
    }
}

#[cfg(feature = "model")]
impl From<WaveformType> for nexrad_model::data::WaveformType {
    fn from(value: WaveformType) -> Self {
        match value {
            WaveformType::ContiguousSurveillance => Self::ContiguousSurveillance,
            WaveformType::ContiguousDopplerWithAmbiguityResolution => {
                Self::ContiguousDopplerWithAmbiguityResolution
            }
            WaveformType::ContiguousDopplerWithoutAmbiguityResolution => {
                Self::ContiguousDopplerWithoutAmbiguityResolution
            }
            WaveformType::Batch => Self::Batch,
            WaveformType::StaggeredPulsePair => Self::StaggeredPulsePair,
            WaveformType::Unknown => Self::Unknown,
        }
    }
}
