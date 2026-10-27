use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Channel configuration (phase coding) for an elevation cut.
///
/// `nexrad_decode::messages::volume_coverage_pattern::ChannelConfiguration` has the same variants
/// and converts to and from this type without loss; it also maps each variant to its ICD code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ChannelConfiguration {
    /// Constant phase.
    ConstantPhase,
    /// Random phase.
    RandomPhase,
    /// SZ2 phase coding.
    Sz2Phase,
    /// A channel configuration the ICD does not define, carrying the code the decoder read.
    Unknown(u8),
}

impl Display for ChannelConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelConfiguration::ConstantPhase => write!(f, "Constant Phase"),
            ChannelConfiguration::RandomPhase => write!(f, "Random Phase"),
            ChannelConfiguration::Sz2Phase => write!(f, "SZ2 Phase"),
            ChannelConfiguration::Unknown(code) => write!(f, "Unknown ({code})"),
        }
    }
}
