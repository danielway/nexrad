use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Radar pulse width configuration.
///
/// `nexrad_decode::messages::volume_coverage_pattern::PulseWidth` has the same variants and
/// converts to and from this type without loss; it also maps each variant to its ICD code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PulseWidth {
    /// Short pulse width.
    Short,
    /// Long pulse width.
    Long,
    /// A pulse width the ICD does not define, carrying the code the decoder read.
    Unknown(u8),
}

impl Display for PulseWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PulseWidth::Short => write!(f, "Short"),
            PulseWidth::Long => write!(f, "Long"),
            PulseWidth::Unknown(code) => write!(f, "Unknown ({code})"),
        }
    }
}
