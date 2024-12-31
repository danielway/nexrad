/// Possible values for the VCP pattern type
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum PatternType {
    Constant,
    Unknown,
}

/// Possible values for pulse width
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum PulseWidth {
    Short,
    Long,
    Unknown,
}

/// Possible values for channel configuration
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ChannelConfiguration {
    ConstantPhase,
    RandomPhase,
    SZ2Phase,
    UnknownPhase,
}

/// Possible values for waveform type
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum WaveformType {
    /// Contiguous Surveillance
    CS,
    /// Contiguous Doppler with Ambiguity Resolution
    CDW,
    /// Contiguous Doppler without Ambiguity Resolution
    CDWO,
    /// Batch
    B,
    /// Staggered Pulse Pair
    SPP,
    Unknown,
}
