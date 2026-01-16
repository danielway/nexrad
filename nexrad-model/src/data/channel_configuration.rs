use std::fmt::{Debug, Display};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Channel configuration (phase coding) for an elevation cut.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ChannelConfiguration {
    /// Constant phase.
    ConstantPhase,
    /// Random phase.
    RandomPhase,
    /// SZ2 phase coding.
    SZ2Phase,
    /// Unknown phase configuration.
    Unknown,
}

impl Display for ChannelConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelConfiguration::ConstantPhase => write!(f, "Constant Phase"),
            ChannelConfiguration::RandomPhase => write!(f, "Random Phase"),
            ChannelConfiguration::SZ2Phase => write!(f, "SZ2 Phase"),
            ChannelConfiguration::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Debug for ChannelConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
