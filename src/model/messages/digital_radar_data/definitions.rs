/// Indicates whether the message is compressed and what type of compression was used.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CompressionIndicator {
    Uncompressed,
    CompressedBZIP2,
    CompressedZLIB,
    FutureUse,
}

/// Possible statuses for a radial describing its position within the larger scan.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RadialStatus {
    ElevationStart,
    IntermediateRadialData,
    ElevationEnd,
    VolumeScanStart,
    VolumeScanEnd,
    /// Start of new elevation which is the last in the VCP.
    ElevationStartVCPFinal,
}
