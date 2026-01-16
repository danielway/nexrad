/// Whether the RDA system has super resolution enabled.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SuperResolutionStatus {
    Enabled,
    Disabled,
    /// Unknown super resolution status value for forward compatibility.
    Unknown(u16),
}
