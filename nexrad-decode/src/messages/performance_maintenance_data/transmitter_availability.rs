/// Transmitter availability status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum TransmitterAvailability {
    /// Transmitter is available.
    Available,
    /// Transmitter is not available.
    NotAvailable,
    /// Unknown status value for forward compatibility.
    Unknown(u16),
}
