/// Antenna servo system status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum ServoStatus {
    /// Servo system is on.
    On,
    /// Servo system is off.
    Off,
    /// Unknown status value for forward compatibility.
    Unknown(u16),
}
