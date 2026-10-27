/// Possible values for the VCP pattern type.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum PatternType {
    /// Constant elevation scan pattern.
    Constant,
    /// A pattern type code the ICD does not define, carrying the raw value.
    Unknown(u16),
}
