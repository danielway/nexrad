/// Indicates whether the message is compressed and what type of compression was used.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum CompressionIndicator {
    Uncompressed,
    CompressedBZIP2,
    CompressedZLIB,
    FutureUse,
}

/// Possible statuses for a radial describing its position within the larger scan.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum RadialStatus {
    ElevationStart,
    IntermediateRadialData,
    ElevationEnd,
    VolumeScanStart,
    VolumeScanEnd,
    /// Start of new elevation which is the last in the VCP.
    ElevationStartVCPFinal,
}

/// Flags indicating special control features.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ControlFlags {
    None,
    RecombinedAzimuthalRadials,
    RecombinedRangeGates,
    RecombinedRadialsAndRangeGatesToLegacyResolution,
}

/// Processing status flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessingStatus {
    RxRNoise,
    CBT,
    Other(u16),
}

/// Volume coverage pattern (VCP) definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VolumeCoveragePattern {
    VCP12,
    VCP31,
    VCP35,
    VCP112,
    VCP212,
    VCP215,
}

/// The value for a data moment/radial, gate, and product. The value may be a floating-point number
/// or a special case such as "below threshold" or "range folded".
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaledMomentValue {
    /// The converted floating-point representation of the data moment value for a gate.
    Value(f32),
    /// The value for this gate was below the signal threshold.
    BelowThreshold,
    /// The value for this gate exceeded the maximum unambiguous range.
    RangeFolded,
}
