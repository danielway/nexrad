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
pub struct MedianFilter {
    /// Kernel width in the azimuth dimension (must be odd, >= 1).
    pub azimuth_kernel: usize,
    /// Kernel width in the range dimension (must be odd, >= 1).
    pub range_kernel: usize,
}

impl SweepProcessor for MedianFilter {
    fn name(&self) -> &str {
        "MedianFilter"
    }

    fn process(&self, input: &SweepField) -> Result<SweepField> {
        if self.azimuth_kernel % 2 == 0 || self.range_kernel % 2 == 0 {
            return Err(Error::InvalidParameter(
                "kernel sizes must be odd".to_string(),
            ));
        }
        if self.azimuth_kernel == 0 || self.range_kernel == 0 {
            return Err(Error::InvalidParameter(
                "kernel sizes must be >= 1".to_string(),
            ));
        }

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
                let (_, status) = input.get(az_idx, gate_idx);
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

                        let (val, st) = input.get(neighbor_az, neighbor_gate as usize);
                        if st == GateStatus::Valid {
                            neighborhood.push(val);
                        }
                    }
                }

                if !neighborhood.is_empty() {
                    neighborhood.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let median = neighborhood[neighborhood.len() / 2];
                    output.set(az_idx, gate_idx, median, GateStatus::Valid);
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
pub struct GaussianSmooth {
    /// Standard deviation in the azimuth dimension (in gate units).
    pub sigma_azimuth: f32,
    /// Standard deviation in the range dimension (in gate units).
    pub sigma_range: f32,
}

impl SweepProcessor for GaussianSmooth {
    fn name(&self) -> &str {
        "GaussianSmooth"
    }

    fn process(&self, input: &SweepField) -> Result<SweepField> {
        if self.sigma_azimuth <= 0.0 || self.sigma_range <= 0.0 {
            return Err(Error::InvalidParameter(
                "sigma values must be positive".to_string(),
            ));
        }

        let az_count = input.azimuth_count();
        let gate_count = input.gate_count();

        // Kernel radius: 3 sigma, clamped to reasonable size
        let az_radius = (self.sigma_azimuth * 3.0).ceil() as usize;
        let range_radius = (self.sigma_range * 3.0).ceil() as usize;

        let mut output = input.clone();

        for az_idx in 0..az_count {
            for gate_idx in 0..gate_count {
                let (_, status) = input.get(az_idx, gate_idx);
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

                        let (val, st) = input.get(neighbor_az, neighbor_gate as usize);
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
                    );
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
        let mut field = SweepField::new_empty(
            "Test",
            "dBZ",
            0.5,
            vec![0.0, 1.0, 2.0, 3.0, 4.0],
            1.0,
            2.0,
            0.25,
            5,
        );

        // Set uniform values
        for az in 0..5 {
            for gate in 0..5 {
                field.set(az, gate, 30.0, GateStatus::Valid);
            }
        }

        field
    }

    #[test]
    fn test_median_filter_uniform() {
        let field = make_test_field();
        let filter = MedianFilter {
            azimuth_kernel: 3,
            range_kernel: 3,
        };

        let result = filter.process(&field).unwrap();

        // Uniform input should produce uniform output
        for az in 0..5 {
            for gate in 0..5 {
                let (val, status) = result.get(az, gate);
                assert_eq!(val, 30.0);
                assert_eq!(status, GateStatus::Valid);
            }
        }
    }

    #[test]
    fn test_median_filter_removes_spike() {
        let mut field = make_test_field();
        // Add a single spike
        field.set(2, 2, 100.0, GateStatus::Valid);

        let filter = MedianFilter {
            azimuth_kernel: 3,
            range_kernel: 3,
        };

        let result = filter.process(&field).unwrap();

        // The spike should be removed (median of mostly 30.0 values)
        let (val, _) = result.get(2, 2);
        assert_eq!(val, 30.0);
    }

    #[test]
    fn test_median_filter_even_kernel_error() {
        let field = make_test_field();
        let filter = MedianFilter {
            azimuth_kernel: 2,
            range_kernel: 3,
        };

        assert!(filter.process(&field).is_err());
    }

    #[test]
    fn test_median_filter_1x1_noop() {
        let mut field = make_test_field();
        field.set(2, 2, 99.0, GateStatus::Valid);

        let filter = MedianFilter {
            azimuth_kernel: 1,
            range_kernel: 1,
        };

        let result = filter.process(&field).unwrap();
        let (val, _) = result.get(2, 2);
        assert_eq!(val, 99.0);
    }

    #[test]
    fn test_median_filter_preserves_nodata() {
        let mut field = make_test_field();
        field.set(2, 2, 0.0, GateStatus::NoData);

        let filter = MedianFilter {
            azimuth_kernel: 3,
            range_kernel: 3,
        };

        let result = filter.process(&field).unwrap();
        let (_, status) = result.get(2, 2);
        assert_eq!(status, GateStatus::NoData);
    }

    #[test]
    fn test_gaussian_smooth_uniform() {
        let field = make_test_field();
        let smoother = GaussianSmooth {
            sigma_azimuth: 1.0,
            sigma_range: 1.0,
        };

        let result = smoother.process(&field).unwrap();

        // Uniform input should produce uniform output
        for az in 0..5 {
            for gate in 0..5 {
                let (val, _) = result.get(az, gate);
                assert!((val - 30.0).abs() < 0.01, "Expected ~30.0, got {}", val);
            }
        }
    }

    #[test]
    fn test_gaussian_smooth_reduces_spike() {
        let mut field = make_test_field();
        field.set(2, 2, 100.0, GateStatus::Valid);

        let smoother = GaussianSmooth {
            sigma_azimuth: 1.0,
            sigma_range: 1.0,
        };

        let result = smoother.process(&field).unwrap();

        // The spike should be reduced (smoothed toward neighbors)
        let (val, _) = result.get(2, 2);
        assert!(val < 100.0, "Expected smoothed value < 100, got {}", val);
        assert!(val > 30.0, "Expected smoothed value > 30, got {}", val);
    }

    #[test]
    fn test_gaussian_smooth_invalid_sigma() {
        let field = make_test_field();
        let smoother = GaussianSmooth {
            sigma_azimuth: 0.0,
            sigma_range: 1.0,
        };

        assert!(smoother.process(&field).is_err());
    }
}
