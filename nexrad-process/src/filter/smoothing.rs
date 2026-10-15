use crate::result::{Error, Result};
use crate::SweepProcessor;
use nexrad_model::data::{GateStatus, SweepField};

/// Median filter in an azimuth x range kernel.
///
/// Replaces each valid gate value with the median of the valid values in its
/// neighborhood. The kernel size is specified as a number of radials in azimuth and
/// a number of gates in range (each must be odd). Gates that are not [`GateStatus::Valid`] (including
/// [`GateStatus::Filtered`]) are excluded from the median computation and left unchanged.
///
/// The azimuth dimension wraps across north only when the field covers the full circle (the
/// gap between its last and first azimuths is at most twice the azimuth spacing); the edges of
/// a sector are boundaries.
///
/// # Example
///
/// ```ignore
/// use nexrad_process::filter::MedianFilter;
///
/// let filter = MedianFilter::new(3, 3)?;
/// let filtered = filter.process(&field)?;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MedianFilter {
    /// Kernel width in the azimuth dimension, in radials (odd, >= 1).
    azimuth_kernel_radials: usize,
    /// Kernel width in the range dimension, in gates (odd, >= 1).
    range_kernel_gates: usize,
}

impl MedianFilter {
    /// Create a median filter with a kernel of `azimuth_kernel_radials` radials by
    /// `range_kernel_gates` gates.
    ///
    /// A 1x1 kernel is valid and leaves the field unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if either kernel size is zero or even.
    pub fn new(azimuth_kernel_radials: usize, range_kernel_gates: usize) -> Result<Self> {
        for (parameter, size) in [
            ("azimuth_kernel_radials", azimuth_kernel_radials),
            ("range_kernel_gates", range_kernel_gates),
        ] {
            if size == 0 || size.is_multiple_of(2) {
                return Err(Error::InvalidParameter {
                    parameter,
                    reason: "must be odd and at least 1",
                });
            }
        }
        Ok(Self {
            azimuth_kernel_radials,
            range_kernel_gates,
        })
    }

    /// Kernel width in the azimuth dimension, in radials.
    pub fn azimuth_kernel_radials(&self) -> usize {
        self.azimuth_kernel_radials
    }

    /// Kernel width in the range dimension, in gates.
    pub fn range_kernel_gates(&self) -> usize {
        self.range_kernel_gates
    }
}

impl SweepProcessor for MedianFilter {
    fn name(&self) -> &str {
        "MedianFilter"
    }

    fn process(&self, input: &SweepField) -> Result<SweepField> {
        // 1x1 kernel is a no-op
        if self.azimuth_kernel_radials == 1 && self.range_kernel_gates == 1 {
            return Ok(input.clone());
        }

        let az_count = input.azimuth_count();
        let gate_count = input.gate_count();
        let az_half = self.azimuth_kernel_radials / 2;
        let range_half = self.range_kernel_gates / 2;

        let wraps = covers_full_circle(input);
        let mut output = input.clone();
        let mut neighborhood =
            Vec::with_capacity(self.azimuth_kernel_radials * self.range_kernel_gates);

        for az_idx in 0..az_count {
            for gate_idx in 0..gate_count {
                let Some((_, status)) = input.get(az_idx, gate_idx) else {
                    continue;
                };
                if status != GateStatus::Valid {
                    continue;
                }

                neighborhood.clear();

                for daz in 0..self.azimuth_kernel_radials {
                    let az_offset = daz as isize - az_half as isize;
                    let Some(neighbor_az) = neighbor_azimuth(az_idx, az_offset, az_count, wraps)
                    else {
                        continue;
                    };

                    for dr in 0..self.range_kernel_gates {
                        let range_offset = dr as isize - range_half as isize;
                        let neighbor_gate = gate_idx as isize + range_offset;

                        if neighbor_gate < 0 || neighbor_gate >= gate_count as isize {
                            continue;
                        }

                        let Some((val, st)) = input.get(neighbor_az, neighbor_gate as usize) else {
                            continue;
                        };
                        if st == GateStatus::Valid {
                            neighborhood.push(val);
                        }
                    }
                }

                if !neighborhood.is_empty() {
                    neighborhood
                        .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    let median = neighborhood[neighborhood.len() / 2];
                    output.set(az_idx, gate_idx, median, GateStatus::Valid)?;
                }
            }
        }

        Ok(output)
    }
}

/// Gaussian smoothing filter in polar coordinates.
///
/// Applies a 2D Gaussian kernel over the azimuth x range grid. The sigma values
/// control the smoothing width in each dimension (in number of gates). Only valid
/// gates contribute to the weighted average; gates that are not [`GateStatus::Valid`]
/// (including [`GateStatus::Filtered`]) are left unchanged.
///
/// The azimuth dimension wraps across north only when the field covers the full circle (the
/// gap between its last and first azimuths is at most twice the azimuth spacing); the edges of
/// a sector are boundaries.
///
/// # Example
///
/// ```ignore
/// use nexrad_process::filter::GaussianSmooth;
///
/// let smoother = GaussianSmooth::new(1.0, 1.0)?;
/// let smoothed = smoother.process(&field)?;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GaussianSmooth {
    /// Standard deviation in the azimuth dimension, in radials.
    sigma_azimuth_radials: f32,
    /// Standard deviation in the range dimension, in gates.
    sigma_range_gates: f32,
}

impl GaussianSmooth {
    /// Create a Gaussian smoother with standard deviations of `sigma_azimuth_radials` radials
    /// in azimuth and `sigma_range_gates` gates in range.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if either sigma is not a finite positive number.
    pub fn new(sigma_azimuth_radials: f32, sigma_range_gates: f32) -> Result<Self> {
        for (parameter, sigma) in [
            ("sigma_azimuth_radials", sigma_azimuth_radials),
            ("sigma_range_gates", sigma_range_gates),
        ] {
            if !(sigma.is_finite() && sigma > 0.0) {
                return Err(Error::InvalidParameter {
                    parameter,
                    reason: "must be finite and greater than zero",
                });
            }
        }
        Ok(Self {
            sigma_azimuth_radials,
            sigma_range_gates,
        })
    }

    /// Standard deviation in the azimuth dimension, in radials.
    pub fn sigma_azimuth_radials(&self) -> f32 {
        self.sigma_azimuth_radials
    }

    /// Standard deviation in the range dimension, in gates.
    pub fn sigma_range_gates(&self) -> f32 {
        self.sigma_range_gates
    }
}

impl SweepProcessor for GaussianSmooth {
    fn name(&self) -> &str {
        "GaussianSmooth"
    }

    fn process(&self, input: &SweepField) -> Result<SweepField> {
        let az_count = input.azimuth_count();
        let gate_count = input.gate_count();

        // Kernel radius: 3 sigma, clamped to reasonable size
        let az_radius = (self.sigma_azimuth_radials * 3.0).ceil() as usize;
        let range_radius = (self.sigma_range_gates * 3.0).ceil() as usize;

        let wraps = covers_full_circle(input);
        let mut output = input.clone();

        for az_idx in 0..az_count {
            for gate_idx in 0..gate_count {
                let Some((_, status)) = input.get(az_idx, gate_idx) else {
                    continue;
                };
                if status != GateStatus::Valid {
                    continue;
                }

                let mut weighted_sum = 0.0f64;
                let mut weight_sum = 0.0f64;

                for daz in 0..=(2 * az_radius) {
                    let az_offset = daz as isize - az_radius as isize;
                    let Some(neighbor_az) = neighbor_azimuth(az_idx, az_offset, az_count, wraps)
                    else {
                        continue;
                    };

                    for dr in 0..=(2 * range_radius) {
                        let range_offset = dr as isize - range_radius as isize;
                        let neighbor_gate = gate_idx as isize + range_offset;

                        if neighbor_gate < 0 || neighbor_gate >= gate_count as isize {
                            continue;
                        }

                        let Some((val, st)) = input.get(neighbor_az, neighbor_gate as usize) else {
                            continue;
                        };
                        if st != GateStatus::Valid {
                            continue;
                        }

                        let az_dist = az_offset as f32;
                        let r_dist = range_offset as f32;
                        let weight = (-0.5
                            * ((az_dist / self.sigma_azimuth_radials).powi(2)
                                + (r_dist / self.sigma_range_gates).powi(2)))
                        .exp() as f64;

                        weighted_sum += val as f64 * weight;
                        weight_sum += weight;
                    }
                }

                if weight_sum > 0.0 {
                    output.set(
                        az_idx,
                        gate_idx,
                        (weighted_sum / weight_sum) as f32,
                        GateStatus::Valid,
                    )?;
                }
            }
        }

        Ok(output)
    }
}

/// Whether `field`'s azimuths cover the full circle: the gap across north between the last
/// and first azimuths is at most twice the azimuth spacing.
fn covers_full_circle(field: &SweepField) -> bool {
    let angles = field.azimuth_angles_degrees();
    let (Some(&first), Some(&last)) = (angles.first(), angles.last()) else {
        return false;
    };
    let wrap_gap = 360.0 - (f64::from(last) - f64::from(first));
    wrap_gap <= 2.0 * f64::from(field.azimuth_spacing_degrees())
}

/// The azimuth index `offset` rows from `az_idx`, wrapping across north when `wraps` and
/// `None` past a sector edge otherwise.
fn neighbor_azimuth(az_idx: usize, offset: isize, az_count: usize, wraps: bool) -> Option<usize> {
    let neighbor = az_idx as isize + offset;
    if wraps {
        Some(neighbor.rem_euclid(az_count as isize) as usize)
    } else {
        usize::try_from(neighbor).ok().filter(|&n| n < az_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_field() -> SweepField {
        let mut field = SweepField::builder("Test", "dBZ")
            .elevation_angle_degrees(0.5)
            .azimuth_angles_degrees(vec![0.0, 1.0, 2.0, 3.0, 4.0])
            .first_gate_range_km(2.0)
            .gate_interval_km(0.25)
            .gate_count(5)
            .build()
            .expect("valid geometry");

        // Set uniform values
        for az in 0..5 {
            for gate in 0..5 {
                field.set(az, gate, 30.0, GateStatus::Valid).unwrap();
            }
        }

        field
    }

    #[test]
    fn test_median_filter_uniform() -> Result<()> {
        let field = make_test_field();
        let filter = MedianFilter::new(3, 3)?;

        let result = filter.process(&field)?;

        // Uniform input should produce uniform output
        for az in 0..5 {
            for gate in 0..5 {
                let (val, status) = result.get(az, gate).unwrap();
                assert_eq!(val, 30.0);
                assert_eq!(status, GateStatus::Valid);
            }
        }
        Ok(())
    }

    #[test]
    fn test_median_filter_removes_spike() -> Result<()> {
        let mut field = make_test_field();
        // Add a single spike
        field.set(2, 2, 100.0, GateStatus::Valid).unwrap();

        let filter = MedianFilter::new(3, 3)?;

        let result = filter.process(&field)?;

        // The spike should be removed (median of mostly 30.0 values)
        let (val, _) = result.get(2, 2).unwrap();
        assert_eq!(val, 30.0);
        Ok(())
    }

    #[test]
    fn test_median_filter_even_kernel_error() {
        assert!(MedianFilter::new(2, 3).is_err());
        assert!(MedianFilter::new(3, 4).is_err());
    }

    #[test]
    fn test_median_filter_zero_kernel_error() {
        assert!(MedianFilter::new(0, 3).is_err());
        assert!(MedianFilter::new(3, 0).is_err());
    }

    #[test]
    fn test_median_filter_1x1_noop() -> Result<()> {
        let mut field = make_test_field();
        field.set(2, 2, 99.0, GateStatus::Valid).unwrap();

        let filter = MedianFilter::new(1, 1)?;

        let result = filter.process(&field)?;
        let (val, _) = result.get(2, 2).unwrap();
        assert_eq!(val, 99.0);
        Ok(())
    }

    #[test]
    fn test_median_filter_preserves_nodata() -> Result<()> {
        let mut field = make_test_field();
        field.set(2, 2, 0.0, GateStatus::NoData).unwrap();

        let filter = MedianFilter::new(3, 3)?;

        let result = filter.process(&field)?;
        let (_, status) = result.get(2, 2).unwrap();
        assert_eq!(status, GateStatus::NoData);
        Ok(())
    }

    #[test]
    fn test_median_filter_ignores_and_preserves_filtered() -> Result<()> {
        let mut field = make_test_field();
        field.set(2, 1, 1000.0, GateStatus::Filtered).unwrap();

        let filter = MedianFilter::new(1, 3)?;

        let result = filter.process(&field)?;
        // With the filtered neighbour included the median at gate 0 would be 1000.
        assert_eq!(result.get(2, 0), Some((30.0, GateStatus::Valid)));
        assert_eq!(result.get(2, 1), Some((1000.0, GateStatus::Filtered)));
        Ok(())
    }

    #[test]
    fn test_gaussian_smooth_ignores_and_preserves_filtered() -> Result<()> {
        let mut field = make_test_field();
        field.set(2, 2, 1000.0, GateStatus::Filtered).unwrap();

        let smoother = GaussianSmooth::new(1.0, 1.0)?;

        let result = smoother.process(&field)?;
        for az in 0..5 {
            for gate in 0..5 {
                let (val, status) = result.get(az, gate).unwrap();
                if (az, gate) == (2, 2) {
                    assert_eq!((val, status), (1000.0, GateStatus::Filtered));
                } else {
                    assert!((val - 30.0).abs() < 0.01, "Expected ~30.0, got {}", val);
                    assert_eq!(status, GateStatus::Valid);
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_gaussian_smooth_uniform() -> Result<()> {
        let field = make_test_field();
        let smoother = GaussianSmooth::new(1.0, 1.0)?;

        let result = smoother.process(&field)?;

        // Uniform input should produce uniform output
        for az in 0..5 {
            for gate in 0..5 {
                let (val, _) = result.get(az, gate).unwrap();
                assert!((val - 30.0).abs() < 0.01, "Expected ~30.0, got {}", val);
            }
        }
        Ok(())
    }

    #[test]
    fn test_gaussian_smooth_reduces_spike() -> Result<()> {
        let mut field = make_test_field();
        field.set(2, 2, 100.0, GateStatus::Valid).unwrap();

        let smoother = GaussianSmooth::new(1.0, 1.0)?;

        let result = smoother.process(&field)?;

        // The spike should be reduced (smoothed toward neighbors)
        let (val, _) = result.get(2, 2).unwrap();
        assert!(val < 100.0, "Expected smoothed value < 100, got {}", val);
        assert!(val > 30.0, "Expected smoothed value > 30, got {}", val);
        Ok(())
    }

    #[test]
    fn test_gaussian_smooth_invalid_sigma() {
        assert!(GaussianSmooth::new(0.0, 1.0).is_err());
        assert!(GaussianSmooth::new(1.0, -1.0).is_err());
        assert!(GaussianSmooth::new(f32::NAN, 1.0).is_err());
        assert!(GaussianSmooth::new(1.0, f32::INFINITY).is_err());
    }

    /// A one-gate field with `azimuth_count` one-degree radials from north, all valid at 30.0.
    fn make_azimuth_field(azimuth_count: usize) -> SweepField {
        let mut field = SweepField::builder("Test", "dBZ")
            .azimuth_angles_degrees((0..azimuth_count).map(|az| az as f32).collect())
            .first_gate_range_km(2.0)
            .gate_interval_km(0.25)
            .gate_count(1)
            .build()
            .expect("valid geometry");
        for az in 0..azimuth_count {
            field.set(az, 0, 30.0, GateStatus::Valid).unwrap();
        }
        field
    }

    #[test]
    fn test_full_circle_rule() {
        assert!(covers_full_circle(&make_azimuth_field(360)));
        // A two-radial gap across north still counts as the full circle.
        assert!(covers_full_circle(&make_azimuth_field(359)));
        assert!(!covers_full_circle(&make_azimuth_field(358)));
        assert!(!covers_full_circle(&make_azimuth_field(90)));
    }

    #[test]
    fn test_gaussian_smooth_sector_does_not_wrap() -> Result<()> {
        let smoother = GaussianSmooth::new(1.0, 1.0)?;

        // A 90-degree sector: the last radial (89 degrees) is not a neighbour of the first.
        let mut sector = make_azimuth_field(90);
        sector.set(89, 0, 100.0, GateStatus::Valid).unwrap();
        let result = smoother.process(&sector)?;
        assert_eq!(result.get(0, 0), Some((30.0, GateStatus::Valid)));

        // Across north in a full circle, 359 degrees is a neighbour of 0 degrees.
        let mut circle = make_azimuth_field(360);
        circle.set(359, 0, 100.0, GateStatus::Valid).unwrap();
        let (value, _) = smoother.process(&circle)?.get(0, 0).unwrap();
        assert!(
            value > 30.0,
            "expected the wrapped neighbour to contribute, got {value}"
        );
        Ok(())
    }

    #[test]
    fn test_median_filter_sector_does_not_wrap() -> Result<()> {
        let filter = MedianFilter::new(3, 1)?;

        // At radial 0 the sector neighbourhood is {10, 20}; wrapping would add 0 from radial 89.
        let mut sector = make_azimuth_field(90);
        sector.set(0, 0, 10.0, GateStatus::Valid).unwrap();
        sector.set(1, 0, 20.0, GateStatus::Valid).unwrap();
        sector.set(89, 0, 0.0, GateStatus::Valid).unwrap();
        assert_eq!(
            filter.process(&sector)?.get(0, 0),
            Some((20.0, GateStatus::Valid))
        );

        let mut circle = make_azimuth_field(360);
        circle.set(0, 0, 10.0, GateStatus::Valid).unwrap();
        circle.set(1, 0, 20.0, GateStatus::Valid).unwrap();
        circle.set(359, 0, 0.0, GateStatus::Valid).unwrap();
        assert_eq!(
            filter.process(&circle)?.get(0, 0),
            Some((10.0, GateStatus::Valid))
        );
        Ok(())
    }

    #[test]
    fn test_parameter_getters() -> Result<()> {
        let median = MedianFilter::new(3, 5)?;
        assert_eq!(median.azimuth_kernel_radials(), 3);
        assert_eq!(median.range_kernel_gates(), 5);

        let gaussian = GaussianSmooth::new(1.5, 2.5)?;
        assert_eq!(gaussian.sigma_azimuth_radials(), 1.5);
        assert_eq!(gaussian.sigma_range_gates(), 2.5);
        Ok(())
    }
}
