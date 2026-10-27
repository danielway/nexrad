/// Transmitter overall summary status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum TransmitterSummaryStatus {
    /// Transmitter is ready.
    Ready,
    /// Transmitter alarm condition.
    Alarm,
    /// Transmitter is in maintenance.
    Maintenance,
    /// Transmitter is recycling.
    Recycle,
    /// Transmitter is preheating.
    Preheat,
    /// Unknown status value for forward compatibility.
    Unknown(u16),
}
