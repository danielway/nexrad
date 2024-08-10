use crate::model::messages::primitive_aliases::Code1;
use std::fmt::{Debug, Formatter};

pub struct SpotBlankingStatus(Code1);

/// Statuses:
///   0 = None
///   1 = Radial
///   2 = Elevation
///   4 = Volume

impl SpotBlankingStatus {
    pub(crate) fn new(code: Code1) -> Self {
        Self(code)
    }

    /// Whether no spot blanking is active.
    pub fn none(&self) -> bool {
        self.0 == 0
    }

    /// Whether spot blanking is active for the radial.
    pub fn radial(&self) -> bool {
        self.0 & 0b0001 != 0
    }

    /// Whether spot blanking is active for the elevation.
    pub fn elevation(&self) -> bool {
        self.0 & 0b0010 != 0
    }

    /// Whether spot blanking is active for the volume.
    pub fn volume(&self) -> bool {
        self.0 & 0b0100 != 0
    }
}

impl Debug for SpotBlankingStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpotBlankingStatus")
            .field("none", &self.none())
            .field("radial", &self.radial())
            .field("elevation", &self.elevation())
            .field("volume", &self.volume())
            .finish()
    }
}
