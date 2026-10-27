/// Radome hatch position.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum RadomeHatch {
    /// Hatch is open.
    Open,
    /// Hatch is closed.
    Closed,
    /// Unknown position value for forward compatibility.
    Unknown(u16),
}
