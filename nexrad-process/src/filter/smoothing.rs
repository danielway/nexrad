use crate::result::{Error, Result};
use crate::SweepProcessor;
use nexrad_model::data::{GateStatus, SweepField};

/// Median filter in an azimuth x range kernel.
///
/// Replaces each valid gate value with the median of the valid values in its
/// neighborhood. The kernel size is specified as the number of gates in each
/// dimension (must be odd). Invalid gates (NoData, BelowThreshold, RangeFolded)
/// are excluded from the median computation and left unchanged.
///
/// The azimuth dimension wraps around 360 degrees.
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
    /// Kernel width in the azimuth dimension (odd, >= 1).
    azimuth_kernel: usize,
    /// Kernel width in the range dimension (odd, >= 1).
    range_kernel: usize,
}

impl MedianFilter {
    /// Create a median filter with the given kernel size in gates.
    ///
    /// A 1x1 kernel is valid and leaves the field unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if either kernel size is zero or even.
    pub fn new(azimuth_kernel: usize, range_kernel: usize) -> Result<Self> {
        if azimuth_kernel == 0 || range_kernel == 0 {
            return Err(Error::InvalidParameter(
                "kernel sizes must be >= 1".to_string(),
            ));
        }
        if azimuth_kernel.is_multiple_of(2) || range_kernel.is_multiple_of(2) {
            return Err(Error::InvalidParameter(
                "kernel sizes must be odd".to_string(),
            ));
        }
        Ok(Self {
            azimuth_kernel,
            range_kernel,
        })
    }

    /// Kernel width in the azimuth dimension, in gates.
    pub fn azimuth_kernel(&self) -> usize {
        self.azimuth_kernel
    }

    /// Kernel width in the range dimension, in gates.
    pub fn range_kernel(&self) -> usize {
        self.range_kernel
    }
}

impl SweepProcessor for MedianFilter {
    fn name(&self) -> &str {
        "MedianFilter"
    }

    fn process(&self, input: &SweepField) -> Result<SweepField> {
        // 1x1 kernel is a no-op
        if self.azimuth_kernel == 1 && self.range_kernel == 1 {
            return Ok(input.clone());
        }

        let az_count = input.azimuth_count();
        let gate_count = input.gate_count();
        let az_half = self.azimuth_kernel / 2;
        let range_half = self.range_kernel / 2;

        let mut output = input.clone();
        let mut neighborhood = Vec::with_capacity(self.azimuth_kernel * self.range_kernel);

        for az_idx in 0..az_count {
            for gate_idx in 0..gate_count {
                let Some((_, status)) = input.get(az_idx, gate_idx) else {
                    continue;
                };
                if status != GateStatus::Valid {
                    continue;
                }

                neighborhood.clear();

                for daz in 0..self.azimuth_kernel {
                    let az_offset = daz as isize - az_half as isize;
                    let neighbor_az =
                        ((az_idx as isize + az_offset).rem_euclid(az_count as isize)) as usize;

                    for dr in 0..self.range_kernel {
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
/// gates contribute to the weighted average; invalid gates are left unchanged.
///
/// The azimuth dimension wraps around 360 degrees.
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
    /// Standard deviation in the azimuth dimension (in gate units).
    sigma_azimuth: f32,
    /// Standard deviation in the range dimension (in gate units).
    sigma_range: f32,
}

impl GaussianSmooth {
    /// Create a Gaussian smoother with the given standard deviations in gate units.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if either sigma is not a finite positive number.
    pub fn new(sigma_azimuth: f32, sigma_range: f32) -> Result<Self> {
        let valid = |sigma: f32| sigma.is_finite() && sigma > 0.0;
        if !valid(sigma_azimuth) || !valid(sigma_range) {
            return Err(Error::InvalidParameter(
                "sigma values must be finite and positive".to_string(),
            ));
        }
        Ok(Self {
            sigma_azimuth,
            sigma_range,
        })
    }

    /// Standard deviation in the azimuth dimension, in gate units.
    pub fn sigma_azimuth(&self) -> f32 {
        self.sigma_azimuth
    }

    /// Standard deviation in the range dimension, in gate units.
    pub fn sigma_range(&self) -> f32 {
        self.sigma_range
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
        let az_radius = (self.sigma_azimuth * 3.0).ceil() as usize;
        let range_radius = (self.sigma_range * 3.0).ceil() as usize;

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
                    let neighbor_az =
                        ((az_idx as isize + az_offset).rem_euclid(az_count as isize)) as usize;

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
                            * ((az_dist / self.sigma_azimuth).powi(2)
                                + (r_dist / self.sigma_range).powi(2)))
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
}
