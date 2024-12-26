/// Possible values for pattern types
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
    CS,
    /// Contiguous Surveillance
    CDW,
    /// Contiguous Doppler w/ Ambiguity Resolution
    CDWO,
    /// Contiguous Doppler w/o Ambiguity Resolution
    B,
    /// Batch
    SPP,
    /// Staggered Pulse Pair
    Unknown,
}
