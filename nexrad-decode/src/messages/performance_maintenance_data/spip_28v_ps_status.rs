/// SPIP 28V power supply status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Spip28vPsStatus {
    /// Power supply has failed.
    Fail,
    /// Power supply is OK.
    Ok,
    /// Unknown status value for forward compatibility.
    Unknown(u16),
}
