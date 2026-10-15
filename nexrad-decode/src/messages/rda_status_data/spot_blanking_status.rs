/// The RDA system's spot blanking status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SpotBlankingStatus {
    /// Spot blanking hardware is not installed.
    NotInstalled,
    /// Spot blanking is enabled.
    Enabled,
    /// Spot blanking is disabled.
    Disabled,
    /// A spot blanking status value the ICD does not define, carrying the raw value.
    Unknown(u16),
}
