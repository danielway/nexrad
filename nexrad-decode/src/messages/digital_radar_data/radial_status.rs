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
