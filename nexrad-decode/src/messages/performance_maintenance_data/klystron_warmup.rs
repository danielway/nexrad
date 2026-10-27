/// Klystron warmup status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum KlystronWarmup {
    /// Normal operation.
    Normal,
    /// Klystron is preheating.
    Preheat,
    /// Unknown status value for forward compatibility.
    Unknown(u16),
}
