/// Generator auto-run/off switch position.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum GeneratorAutoRunOffSwitch {
    /// Switch is in manual position.
    Manual,
    /// Switch is in auto position.
    Auto,
    /// Unknown position value for forward compatibility.
    Unknown(u16),
}
