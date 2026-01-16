/// The possible RDA system operability statuses.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum OperabilityStatus {
    OnLine,
    MaintenanceActionRequired,
    MaintenanceActionMandatory,
    CommandedShutDown,
    Inoperable,
    /// Unknown operability status value for forward compatibility.
    Unknown(u16),
}
