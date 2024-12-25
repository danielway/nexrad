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

