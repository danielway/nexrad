/// The RDA system's RMS control status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum RmsControlStatus {
    /// Not an RMS-controlled site.
    NonRms,
    /// RMS (Remote Monitoring System) is in control.
    RmsInControl,
    /// RDA is in local control.
    RdaInControl,
    /// Unknown RMS control status value for forward compatibility.
    Unknown(u16),
}
