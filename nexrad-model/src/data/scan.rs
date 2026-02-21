use crate::data::{Sweep, VolumeCoveragePattern};
use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single radar scan composed of a series of sweeps. This represents a single volume scan which
/// is composed of multiple sweeps at different elevations. The pattern of sweeps, including
/// elevations and resolution, is determined by the scanning strategy of the radar. This is
/// referred to as the Volume Coverage Pattern.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Scan {
    coverage_pattern: VolumeCoveragePattern,
    sweeps: Vec<Sweep>,
}

impl Scan {
    /// Create a new radar scan with the given volume coverage pattern and sweeps.
    pub fn new(coverage_pattern: VolumeCoveragePattern, sweeps: Vec<Sweep>) -> Self {
        Self {
            coverage_pattern,
            sweeps,
        }
    }

    /// This scan's volume coverage pattern number.
    pub fn coverage_pattern_number(&self) -> u16 {
        self.coverage_pattern.pattern_number()
    }

    /// The volume coverage pattern configuration for this scan.
    ///
    /// This contains detailed configuration about how the radar scans the volume, including
    /// per-elevation settings, SAILS/MRLE/MPDA capabilities, and various thresholds.
    pub fn coverage_pattern(&self) -> &VolumeCoveragePattern {
        &self.coverage_pattern
    }

    /// The elevation sweeps comprising this scan.
    pub fn sweeps(&self) -> &[Sweep] {
        &self.sweeps
    }
}

impl Display for Scan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Scan (VCP {}, {} sweeps)",
            self.coverage_pattern.pattern_number(),
            self.sweeps.len()
        )
    }
}
