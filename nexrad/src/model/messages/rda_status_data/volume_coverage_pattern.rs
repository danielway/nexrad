use crate::model::messages::primitive_aliases::SInteger2;
use std::fmt::Debug;

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

impl Debug for VolumeCoveragePatternNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VolumeCoveragePatternNumber")
            .field("number", &self.number())
            .field("local", &self.local())
            .field("remote", &self.remote())
            .finish()
    }
}
