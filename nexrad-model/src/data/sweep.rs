use crate::data::{Product, Radial, SweepField};
use crate::result::{Error, Result};
use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

#[cfg(feature = "uom")]
use uom::si::f64::Angle;

/// A single radar sweep composed of a series of radials. This represents a full rotation of the
/// radar at some elevation angle and contains the Level II data (reflectivity, velocity, and
/// spectrum width) for each azimuth angle in that sweep. The resolution of the sweep dictates the
/// azimuthal distance between rays and thus and number of rays in the sweep. Multiple sweeps are
/// taken at different elevation angles to create a complete scan.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sweep {
    elevation_number: u8,
    radials: Vec<Radial>,
}

impl Sweep {
    /// Create a new radar sweep with the given elevation number and radials.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ElevationMismatch`] if any radial's elevation number differs from
    /// `elevation_number`.
    pub fn new(elevation_number: u8, radials: Vec<Radial>) -> Result<Self> {
        if let Some(radial) = radials
            .iter()
            .find(|radial| radial.elevation_number() != elevation_number)
        {
            return Err(Error::ElevationMismatch {
                expected: elevation_number,
                actual: radial.elevation_number(),
            });
        }

        Ok(Self {
            elevation_number,
            radials,
        })
    }

    /// Create a new radar sweep from a list of radials by splitting them by elevation.
    ///
    /// Consecutive radials with the same elevation number form one sweep, so every returned
    /// sweep upholds the invariant checked by [`new`](Self::new).
    pub fn from_radials(radials: Vec<Radial>) -> Vec<Self> {
        let mut sweeps = Vec::new();

        let mut sweep_elevation_number = None;
        let mut sweep_radials = Vec::new();

        for radial in radials {
            if let Some(elevation_number) = sweep_elevation_number {
                if elevation_number != radial.elevation_number() {
                    sweeps.push(Sweep {
                        elevation_number,
                        radials: std::mem::take(&mut sweep_radials),
                    });
                }
            }

            sweep_elevation_number = Some(radial.elevation_number());
            sweep_radials.push(radial);
        }

        // Push the final sweep if there are remaining radials
        if let Some(elevation_number) = sweep_elevation_number {
            if !sweep_radials.is_empty() {
                sweeps.push(Sweep {
                    elevation_number,
                    radials: sweep_radials,
                });
            }
        }

        sweeps
    }

    /// The index number for this radial's elevation in the scan. The precise elevation angle
    /// varies and can be found in individual radials.
    pub fn elevation_number(&self) -> u8 {
        self.elevation_number
    }

    /// The elevation angle in degrees for this sweep, derived from the median of all radial
    /// elevation angles.
    ///
    /// The median is used rather than the first radial because radials at the beginning and end
    /// of a sweep may report transitional elevation angles as the antenna moves between sweeps.
    /// The median is robust against these outliers.
    ///
    /// Returns `None` if the sweep has no radials.
    pub fn elevation_angle_degrees(&self) -> Option<f32> {
        let mut angles: Vec<f32> = self
            .radials
            .iter()
            .map(|r| r.elevation_angle_degrees())
            .collect();
        median(&mut angles)
    }

    /// The elevation angle for this sweep.
    ///
    /// Uses the median of all radial elevation angles for robustness against transitional
    /// radials at sweep boundaries. See [`elevation_angle_degrees`](Self::elevation_angle_degrees)
    /// for details.
    ///
    /// Returns `None` if the sweep has no radials.
    #[cfg(feature = "uom")]
    pub fn elevation_angle(&self) -> Option<Angle> {
        self.elevation_angle_degrees()
            .map(|degrees| Angle::new::<uom::si::angle::degree>(degrees as f64))
    }

    /// The radials comprising this sweep.
    pub fn radials(&self) -> &[Radial] {
        &self.radials
    }

    /// Extract this sweep's data for `product` as a [`SweepField`].
    ///
    /// Returns `None` if no radial in the sweep carries data for `product`. This is
    /// equivalent to [`SweepField::from_radials`] over [`radials`](Self::radials).
    ///
    /// # Example
    ///
    /// ```
    /// use nexrad_model::data::{MomentData, Product, Radial, Sweep};
    ///
    /// let reflectivity = MomentData::from_fixed_point(
    ///     Product::Reflectivity, 3, 2000, 250, 8, 2.0, 66.0, vec![100, 110, 120],
    /// )?;
    /// let radial = Radial::builder(0, 0.0, 1, 0.5).moment(reflectivity).build()?;
    /// let sweep = Sweep::new(1, vec![radial])?;
    ///
    /// let field = sweep.field(Product::Reflectivity).unwrap();
    /// assert_eq!(field.gate_count(), 3);
    /// assert!(sweep.field(Product::Velocity).is_none());
    /// # Ok::<(), nexrad_model::result::Error>(())
    /// ```
    pub fn field(&self, product: Product) -> Option<SweepField> {
        SweepField::from_radials(self.radials(), product)
    }

    /// The time range of this sweep as `(earliest, latest)` collection times.
    ///
    /// Returns `None` if the sweep has no radials or no valid timestamps.
    #[cfg(feature = "chrono")]
    pub fn time_range(&self) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
        let mut earliest: Option<DateTime<Utc>> = None;
        let mut latest: Option<DateTime<Utc>> = None;

        for radial in &self.radials {
            if let Some(time) = radial.collection_time() {
                earliest = Some(match earliest {
                    Some(e) => e.min(time),
                    None => time,
                });
                latest = Some(match latest {
                    Some(l) => l.max(time),
                    None => time,
                });
            }
        }

        earliest.zip(latest)
    }

    /// Merges this sweep with another sweep, combining their radials into a single sweep. The
    /// sweeps must be at the same elevation, and they should not have duplicate azimuth radials.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ElevationMismatch`] if the sweeps' elevation numbers differ.
    pub fn merge(self, other: Self) -> Result<Self> {
        if self.elevation_number != other.elevation_number {
            return Err(Error::ElevationMismatch {
                expected: self.elevation_number,
                actual: other.elevation_number,
            });
        }

        let mut radials = self.radials;
        radials.extend(other.radials);
        radials.sort_by_key(|radial| radial.azimuth_number());

        Ok(Self {
            elevation_number: self.elevation_number,
            radials,
        })
    }
}

impl Display for Sweep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let (Some(first), Some(last)) = (self.radials.first(), self.radials.last()) {
            write!(
                f,
                "Sweep ({:.1}-{:.1} deg, {} radials, {} deg spacing)",
                first.azimuth_angle_degrees(),
                last.azimuth_angle_degrees(),
                self.radials.len(),
                first.azimuth_spacing_degrees()
            )
        } else {
            write!(f, "Sweep (no radials)")
        }
    }
}

/// The median of `values`, sorting them in place. The mean of the two middle values is used for
/// an even count. Returns `None` if `values` is empty.
pub(crate) fn median(values: &mut [f32]) -> Option<f32> {
    values.sort_by(f32::total_cmp);

    let mid = values.len() / 2;
    let upper = *values.get(mid)?;
    if values.len().is_multiple_of(2) {
        let lower = *values.get(mid.checked_sub(1)?)?;
        Some((lower + upper) / 2.0)
    } else {
        Some(upper)
    }
}
