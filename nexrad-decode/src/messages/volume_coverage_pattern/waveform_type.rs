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
