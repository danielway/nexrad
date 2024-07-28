use crate::data::Sweep;
use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single radar scan composed of a series of sweeps. This represents a single volume scan which
/// is composed of multiple sweeps at different elevations. The pattern of sweeps, including
/// elevations and resolution, is determined by the scanning strategy of the radar. This is
/// referred to as the Volume Coverage Pattern.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Scan {
    coverage_pattern_number: u16,
    sweeps: Vec<Sweep>,
}

impl Scan {
    /// Create a new radar scan with the given coverage pattern number and sweeps.
    pub fn new(coverage_pattern_number: u16, sweeps: Vec<Sweep>) -> Self {
        Self {
            coverage_pattern_number,
            sweeps,
        }
    }

    /// This scan's volume coverage pattern number.
    pub fn coverage_pattern_number(&self) -> u16 {
        self.coverage_pattern_number
    }

    /// The elevation sweeps comprising this scan.
    pub fn sweeps(&self) -> &Vec<Sweep> {
        self.sweeps.as_ref()
    }
}

impl Debug for Scan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scan")
            .field("coverage_pattern_number", &self.coverage_pattern_number())
            .field("sweeps", &self.sweeps())
            .finish()
    }
}
