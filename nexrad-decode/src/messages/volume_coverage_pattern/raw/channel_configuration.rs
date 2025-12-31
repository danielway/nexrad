/// Possible values for channel configuration
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ChannelConfiguration {
    ConstantPhase,
    RandomPhase,
    SZ2Phase,
    UnknownPhase,
}
