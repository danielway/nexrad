/// Flags indicating special control features.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ControlFlags {
    None,
    RecombinedAzimuthalRadials,
    RecombinedRangeGates,
    RecombinedRadialsAndRangeGatesToLegacyResolution,
    /// Unknown control flag value for forward compatibility.
    Unknown(u8),
}
