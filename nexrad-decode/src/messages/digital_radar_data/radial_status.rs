/// Possible statuses for a radial describing its position within the larger scan.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum RadialStatus {
    /// First radial of an elevation sweep (not the first sweep in the volume).
    ElevationStart,
    /// A radial within an elevation sweep (not the first or last radial).
    IntermediateRadialData,
    /// Last radial of an elevation sweep.
    ElevationEnd,
    /// First radial of the first elevation sweep in a volume scan.
    VolumeScanStart,
    /// Last radial of the last elevation sweep in a volume scan.
    VolumeScanEnd,
    /// Start of new elevation which is the last in the VCP.
    ElevationStartVcpFinal,
    /// An unrecognized radial status value.
    Unknown(u8),
}

impl RadialStatus {
    /// Decodes a radial status code as it appears on the wire.
    pub(crate) fn from_raw(value: u8) -> Self {
        match value {
            0 => RadialStatus::ElevationStart,
            1 => RadialStatus::IntermediateRadialData,
            2 => RadialStatus::ElevationEnd,
            3 => RadialStatus::VolumeScanStart,
            4 => RadialStatus::VolumeScanEnd,
            5 => RadialStatus::ElevationStartVcpFinal,
            other => RadialStatus::Unknown(other),
        }
    }
}

#[cfg(feature = "model")]
impl From<RadialStatus> for nexrad_model::data::RadialStatus {
    fn from(status: RadialStatus) -> Self {
        use nexrad_model::data::RadialStatus as ModelRadialStatus;

        match status {
            RadialStatus::ElevationStart => ModelRadialStatus::ElevationStart,
            RadialStatus::IntermediateRadialData => ModelRadialStatus::IntermediateRadialData,
            RadialStatus::ElevationEnd => ModelRadialStatus::ElevationEnd,
            RadialStatus::VolumeScanStart => ModelRadialStatus::ScanStart,
            RadialStatus::VolumeScanEnd => ModelRadialStatus::ScanEnd,
            RadialStatus::ElevationStartVcpFinal => ModelRadialStatus::ElevationStartVcpFinal,
            RadialStatus::Unknown(v) => ModelRadialStatus::Unknown(v),
        }
    }
}
