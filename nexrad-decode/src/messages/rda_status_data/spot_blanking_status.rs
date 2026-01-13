/// Indicates whether this is the RDA system's controlling channel.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SpotBlankingStatus {
    NotInstalled,
    Enabled,
    Disabled,
}
