use crate::result::{Error, Result};
use crate::SweepProcessor;
use nexrad_model::data::{GateStatus, SweepField};

/// Masks gates whose values fall outside a specified range.
///
/// Valid gates with values below `min` or above `max` are marked [`GateStatus::Filtered`] and
/// keep their values. Either bound may be `None` to leave that side unbounded. Gates that are not
/// [`GateStatus::Valid`] pass through unchanged.
///
/// # Example
///
/// ```
/// use nexrad_process::filter::ThresholdFilter;
/// use nexrad_process::SweepProcessor;
/// # let field = nexrad_model::data::SweepField::builder("Reflectivity", "dBZ")
/// #     .product(nexrad_model::data::Product::Reflectivity)
/// #     .elevation_angle_degrees(0.5)
/// #     .azimuth_angles_degrees((0..360).map(|a| a as f32).collect())
/// #     .first_gate_range_km(2.0)
/// #     .gate_interval_km(0.25)
/// #     .gate_count(400)
/// #     .build()?;
///
/// // Remove all reflectivity below 5 dBZ
/// let filter = ThresholdFilter::new(Some(5.0), None)?;
/// let filtered = filter.process(&field)?;
/// # Ok::<(), nexrad_process::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ThresholdFilter {
    /// Minimum acceptable value. Gates below this are masked.
    min: Option<f32>,
    /// Maximum acceptable value. Gates above this are masked.
    max: Option<f32>,
}

impl ThresholdFilter {
    /// Create a threshold filter with optional lower and upper bounds.
    ///
    /// Passing `None` for both bounds yields a filter that leaves every gate unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if either bound is NaN or if `min > max`.
    pub fn new(min: Option<f32>, max: Option<f32>) -> Result<Self> {
        for (parameter, bound) in [("min", min), ("max", max)] {
            if bound.is_some_and(f32::is_nan) {
                return Err(Error::InvalidParameter {
                    parameter,
                    reason: "must not be NaN",
                });
            }
        }
        if let (Some(lo), Some(hi)) = (min, max) {
            if lo > hi {
                return Err(Error::InvalidParameter {
                    parameter: "min",
                    reason: "must not exceed max",
                });
            }
        }
        Ok(Self { min, max })
    }

    /// Minimum acceptable value, if bounded below.
    pub fn min(&self) -> Option<f32> {
        self.min
    }

    /// Maximum acceptable value, if bounded above.
    pub fn max(&self) -> Option<f32> {
        self.max
    }
}

impl SweepProcessor for ThresholdFilter {
    fn name(&self) -> &str {
        "ThresholdFilter"
    }

    fn process(&self, input: &SweepField) -> Result<SweepField> {
        let mut output = input.clone();

        for az_idx in 0..output.azimuth_count() {
            for gate_idx in 0..output.gate_count() {
                let Some((val, status)) = output.get(az_idx, gate_idx) else {
                    continue;
                };

                if status == GateStatus::Valid {
                    let below_min = self.min.is_some_and(|m| val < m);
                    let above_max = self.max.is_some_and(|m| val > m);

                    if below_min || above_max {
                        output.set(az_idx, gate_idx, val, GateStatus::Filtered)?;
                    }
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SweepProcessor;

    fn make_test_field() -> SweepField {
        let mut field = SweepField::builder("Test", "dBZ")
            .elevation_angle_degrees(0.5)
            .azimuth_angles_degrees(vec![0.0, 1.0, 2.0])
            .first_gate_range_km(2.0)
            .gate_interval_km(0.25)
            .gate_count(5)
            .build()
            .expect("valid geometry");

        // Set some test values
        for az in 0..3 {
            for gate in 0..5 {
                let value = (az * 5 + gate) as f32 * 5.0; // 0, 5, 10, ..., 70
                field.set(az, gate, value, GateStatus::Valid).unwrap();
            }
        }

        field
    }

    #[test]
    fn test_threshold_min_only() -> Result<()> {
        let field = make_test_field();
        let filter = ThresholdFilter::new(Some(20.0), None)?;

        let result = filter.process(&field)?;

        // Values below 20 should be masked
        for az in 0..3 {
            for gate in 0..5 {
                let original_value = (az * 5 + gate) as f32 * 5.0;
                let (val, status) = result.get(az, gate).unwrap();
                assert_eq!(val, original_value);
                if original_value < 20.0 {
                    assert_eq!(status, GateStatus::Filtered);
                } else {
                    assert_eq!(status, GateStatus::Valid);
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_threshold_max_only() -> Result<()> {
        let field = make_test_field();
        let filter = ThresholdFilter::new(None, Some(40.0))?;

        let result = filter.process(&field)?;

        for az in 0..3 {
            for gate in 0..5 {
                let original_value = (az * 5 + gate) as f32 * 5.0;
                let (_, status) = result.get(az, gate).unwrap();
                if original_value > 40.0 {
                    assert_eq!(status, GateStatus::Filtered);
                } else {
                    assert_eq!(status, GateStatus::Valid);
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_threshold_both_bounds() -> Result<()> {
        let field = make_test_field();
        let filter = ThresholdFilter::new(Some(15.0), Some(50.0))?;

        let result = filter.process(&field)?;

        for az in 0..3 {
            for gate in 0..5 {
                let original_value = (az * 5 + gate) as f32 * 5.0;
                let (_, status) = result.get(az, gate).unwrap();
                if !(15.0..=50.0).contains(&original_value) {
                    assert_eq!(status, GateStatus::Filtered);
                } else {
                    assert_eq!(status, GateStatus::Valid);
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_threshold_preserves_nodata() -> Result<()> {
        let mut field = make_test_field();
        // Mark some gates as already NoData
        field.set(1, 2, 0.0, GateStatus::NoData).unwrap();

        let filter = ThresholdFilter::new(Some(0.0), None)?;

        let result = filter.process(&field)?;

        // The NoData gate should remain NoData
        let (_, status) = result.get(1, 2).unwrap();
        assert_eq!(status, GateStatus::NoData);
        Ok(())
    }

    #[test]
    fn test_threshold_preserves_filtered() -> Result<()> {
        let mut field = make_test_field();
        field.set(1, 2, 35.0, GateStatus::Filtered).unwrap();

        let filter = ThresholdFilter::new(None, Some(20.0))?;

        let result = filter.process(&field)?;

        // An already-filtered gate keeps its value and status.
        assert_eq!(result.get(1, 2), Some((35.0, GateStatus::Filtered)));
        Ok(())
    }

    #[test]
    fn test_threshold_rejects_nan_bounds() {
        assert!(ThresholdFilter::new(Some(f32::NAN), None).is_err());
        assert!(ThresholdFilter::new(None, Some(f32::NAN)).is_err());
    }

    #[test]
    fn test_threshold_rejects_inverted_bounds() {
        assert!(ThresholdFilter::new(Some(50.0), Some(10.0)).is_err());
    }

    #[test]
    fn test_threshold_accepts_equal_bounds() -> Result<()> {
        let filter = ThresholdFilter::new(Some(10.0), Some(10.0))?;
        assert_eq!(filter.min(), Some(10.0));
        assert_eq!(filter.max(), Some(10.0));
        Ok(())
    }

    #[test]
    fn test_threshold_no_bounds() -> Result<()> {
        let field = make_test_field();
        let filter = ThresholdFilter::new(None, None)?;

        let result = filter.process(&field)?;

        // All valid values should remain valid
        for az in 0..3 {
            for gate in 0..5 {
                let (orig_val, _) = field.get(az, gate).unwrap();
                let (result_val, result_status) = result.get(az, gate).unwrap();
                assert_eq!(result_val, orig_val);
                assert_eq!(result_status, GateStatus::Valid);
            }
        }
        Ok(())
    }
}
