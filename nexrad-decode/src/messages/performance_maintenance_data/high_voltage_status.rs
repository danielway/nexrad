/// Transmitter high voltage status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum HighVoltageStatus {
    /// High voltage is on.
    On,
    /// High voltage is off.
    Off,
    /// Unknown status value for forward compatibility.
    Unknown(u16),
}
