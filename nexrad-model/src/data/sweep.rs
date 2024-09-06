use crate::data::Radial;
use crate::result::{Error, Result};
use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single radar sweep composed of a series of radials. This represents a full rotation of the
/// radar at some elevation angle and contains the Level II data (reflectivity, velocity, and
/// spectrum width) for each azimuth angle in that sweep. The resolution of the sweep dictates the
/// azimuthal distance between rays and thus and number of rays in the sweep. Multiple sweeps are
/// taken at different elevation angles to create a volume scan.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sweep {
    elevation_number: u8,
    radials: Vec<Radial>,
}

impl Sweep {
    /// Create a new radar sweep with the given elevation number and radials.
    pub fn new(elevation_number: u8, radials: Vec<Radial>) -> Self {
        Self {
            elevation_number,
            radials,
        }
    }

    /// Create a new radar sweep from a list of radials by splitting them by elevation.
    pub fn from_radials(radials: Vec<Radial>) -> Vec<Self> {
        let mut sweeps = Vec::new();

        let mut sweep_elevation_number = None;
        let mut sweep_radials = Vec::new();

        for radial in radials {
            if let Some(elevation_number) = sweep_elevation_number {
                if elevation_number != radial.elevation_number() {
                    sweeps.push(Sweep::new(elevation_number, sweep_radials));
                    sweep_radials = Vec::new();
                }
            }

            sweep_elevation_number = Some(radial.elevation_number());
            sweep_radials.push(radial);
        }

        sweeps
    }

    /// The index number for this radial's elevation in the volume scan. The precise elevation angle
    /// varies and can be found in individual radials.
    pub fn elevation_number(&self) -> u8 {
        self.elevation_number
    }

    /// The radials comprising this sweep.
    pub fn radials(&self) -> &Vec<Radial> {
        self.radials.as_ref()
    }

    /// Merges this sweep with another sweep, combining their radials into a single sweep. The
    /// sweeps must be at the same elevation, and they should not have duplicate azimuth radials.
    pub fn merge(self, other: Self) -> Result<Self> {
        if self.elevation_number != other.elevation_number {
            return Err(Error::ElevationMismatchError);
        }

        let mut radials = self.radials;
        radials.extend(other.radials);
        radials.sort_by(|a, b| a.azimuth_number().cmp(&b.azimuth_number()));

        Ok(Self {
            elevation_number: self.elevation_number,
            radials,
        })
    }
}

impl Debug for Sweep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sweep")
            .field("elevation_number", &self.elevation_number())
            .field("radials", &self.radials())
            .finish()
    }
}
