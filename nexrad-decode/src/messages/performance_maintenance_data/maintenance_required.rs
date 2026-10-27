/// Transmitter maintenance required status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum MaintenanceRequired {
    /// Maintenance is not required.
    No,
    /// Maintenance is required.
    Required,
    /// Unknown status value for forward compatibility.
    Unknown(u16),
}
