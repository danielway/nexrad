/// AME A/D converter status.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum AmeAdConverterStatus {
    /// A/D converter is operating normally.
    Ok,
    /// A/D converter has failed.
    Fail,
    /// Unknown status value for forward compatibility.
    Unknown(u16),
}
