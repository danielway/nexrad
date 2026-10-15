/// Possible values for pulse width.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum PulseWidth {
    /// Short pulse width for higher resolution.
    Short,
    /// Long pulse width for extended range.
    Long,
    /// A pulse width code the ICD does not define, carrying the raw value.
    Unknown(u8),
}

#[cfg(feature = "model")]
impl From<PulseWidth> for nexrad_model::data::PulseWidth {
    fn from(value: PulseWidth) -> Self {
        match value {
            PulseWidth::Short => Self::Short,
            PulseWidth::Long => Self::Long,
            PulseWidth::Unknown(_) => Self::Unknown,
        }
    }
}
