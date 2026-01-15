/// Flags indicating special control features.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ControlFlags {
    None,
    RecombinedAzimuthalRadials,
    RecombinedRangeGates,
    RecombinedRadialsAndRangeGatesToLegacyResolution,
}
