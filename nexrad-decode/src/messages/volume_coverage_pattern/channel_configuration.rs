/// Possible values for channel configuration.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ChannelConfiguration {
    /// Constant phase processing.
    ConstantPhase,
    /// Random phase processing.
    RandomPhase,
    /// SZ-2 phase coding for range dealiasing.
    Sz2Phase,
    /// Unknown phase configuration.
    UnknownPhase,
}

#[cfg(feature = "model")]
impl From<ChannelConfiguration> for nexrad_model::data::ChannelConfiguration {
    fn from(value: ChannelConfiguration) -> Self {
        match value {
            ChannelConfiguration::ConstantPhase => Self::ConstantPhase,
            ChannelConfiguration::RandomPhase => Self::RandomPhase,
            ChannelConfiguration::Sz2Phase => Self::Sz2Phase,
            ChannelConfiguration::UnknownPhase => Self::Unknown,
        }
    }
}
