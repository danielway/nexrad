use std::fmt::{Display, Formatter};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Channel configuration (phase coding) of an elevation cut, as the RDA/RPG ICD codes it.
///
/// This type has the same variants as `nexrad_model::data::ChannelConfiguration`, and the
/// `From` impls between them (feature `model`) convert every value, including
/// [`Unknown`](Self::Unknown) with its code. This copy also knows the ICD code for each variant:
/// see [`from_number`](Self::from_number) and [`number`](Self::number).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ChannelConfiguration {
    /// Constant phase (code 0).
    ConstantPhase,
    /// Random phase (code 1).
    RandomPhase,
    /// SZ2 phase (code 2).
    Sz2Phase,
    /// A channel configuration code the ICD does not define, carrying the code.
    Unknown(u8),
}

impl ChannelConfiguration {
    /// Classifies a channel configuration code as it appears on the wire. Codes the ICD does not
    /// define map to [`ChannelConfiguration::Unknown`].
    pub fn from_number(code: u8) -> Self {
        match code {
            0 => ChannelConfiguration::ConstantPhase,
            1 => ChannelConfiguration::RandomPhase,
            2 => ChannelConfiguration::Sz2Phase,
            other => ChannelConfiguration::Unknown(other),
        }
    }

    /// The channel configuration code as it appears on the wire.
    pub fn number(self) -> u8 {
        match self {
            ChannelConfiguration::ConstantPhase => 0,
            ChannelConfiguration::RandomPhase => 1,
            ChannelConfiguration::Sz2Phase => 2,
            ChannelConfiguration::Unknown(code) => code,
        }
    }
}

impl Display for ChannelConfiguration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelConfiguration::ConstantPhase => write!(f, "Constant Phase"),
            ChannelConfiguration::RandomPhase => write!(f, "Random Phase"),
            ChannelConfiguration::Sz2Phase => write!(f, "SZ2 Phase"),
            ChannelConfiguration::Unknown(code) => write!(f, "Unknown ({code})"),
        }
    }
}

#[cfg(feature = "model")]
impl From<ChannelConfiguration> for nexrad_model::data::ChannelConfiguration {
    fn from(value: ChannelConfiguration) -> Self {
        match value {
            ChannelConfiguration::ConstantPhase => Self::ConstantPhase,
            ChannelConfiguration::RandomPhase => Self::RandomPhase,
            ChannelConfiguration::Sz2Phase => Self::Sz2Phase,
            ChannelConfiguration::Unknown(code) => Self::Unknown(code),
        }
    }
}

#[cfg(feature = "model")]
impl From<nexrad_model::data::ChannelConfiguration> for ChannelConfiguration {
    fn from(value: nexrad_model::data::ChannelConfiguration) -> Self {
        use nexrad_model::data::ChannelConfiguration as Model;

        match value {
            Model::ConstantPhase => Self::ConstantPhase,
            Model::RandomPhase => Self::RandomPhase,
            Model::Sz2Phase => Self::Sz2Phase,
            Model::Unknown(code) => Self::Unknown(code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChannelConfiguration;

    #[test]
    fn number_round_trips_for_every_code() {
        for code in 0..=u8::MAX {
            assert_eq!(ChannelConfiguration::from_number(code).number(), code);
        }
        assert_eq!(
            ChannelConfiguration::from_number(2),
            ChannelConfiguration::Sz2Phase
        );
        assert_eq!(
            ChannelConfiguration::from_number(3),
            ChannelConfiguration::Unknown(3)
        );
    }

    #[test]
    fn displays_long_form() {
        assert_eq!(ChannelConfiguration::Sz2Phase.to_string(), "SZ2 Phase");
        assert_eq!(ChannelConfiguration::Unknown(3).to_string(), "Unknown (3)");
    }

    #[cfg(feature = "model")]
    #[test]
    fn converts_to_and_from_the_model_without_loss() {
        for code in 0..=u8::MAX {
            let decoded = ChannelConfiguration::from_number(code);
            let model = nexrad_model::data::ChannelConfiguration::from(decoded);
            assert_eq!(ChannelConfiguration::from(model), decoded);
        }
    }
}
