/// The possible RDA system transition power source statuses.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TransitionPowerSourceStatus {
    NotInstalled,
    Off,
    OK,
    Unknown,
    /// Unrecognized TPS status value for forward compatibility.
    Other(u16),
}
