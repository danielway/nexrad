use crate::model::primitive_aliases::SInteger2;

/// The RDA system's volume coverage pattern number.
pub struct VolumeCoveragePatternNumber(SInteger2);

impl VolumeCoveragePatternNumber {
    pub(crate) fn new(value: SInteger2) -> Self {
        Self(value)
    }

    /// The volume coverage pattern number.
    pub fn number(&self) -> i16 {
        self.0.abs()
    }

    /// Whether the volume coverage pattern number was specified locally.
    pub fn local(&self) -> bool {
        self.0 < 0
    }

    /// Whether the volume coverage pattern number was specified remotely.
    pub fn remote(&self) -> bool {
        self.0 > 0
    }
}
