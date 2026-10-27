/// The possible RDA system transition power source statuses.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum TransitionPowerSourceStatus {
    /// Transition power source is not installed.
    NotInstalled,
    /// Transition power source is off.
    Off,
    /// Transition power source is OK.
    Ok,
    /// The transition power source status is unknown (code 4), or a value the ICD does not
    /// define, carrying the raw value.
    Unknown(u16),
}
