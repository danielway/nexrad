use crate::result::Result;
use crate::SweepProcessor;
use nexrad_model::data::{GateStatus, SweepField};

/// Masks gates whose values fall outside a specified range.
///
/// Gates with values below `min` or above `max` are set to [`GateStatus::NoData`].
/// Either bound may be `None` to leave that side unbounded.
///
/// # Example
///
/// ```ignore
/// use nexrad_process::filter::ThresholdFilter;
///
/// // Remove all reflectivity below 5 dBZ
/// let filter = ThresholdFilter { min: Some(5.0), max: None };
/// let filtered = filter.process(&field)?;
/// ```
pub struct ThresholdFilter {
    /// Minimum acceptable value. Gates below this are masked.
    pub min: Option<f32>,
    /// Maximum acceptable value. Gates above this are masked.
    pub max: Option<f32>,
}

impl SweepProcessor for ThresholdFilter {
    fn name(&self) -> &str {
        "ThresholdFilter"
    }

    fn process(&self, input: &SweepField) -> Result<SweepField> {
        let mut output = input.clone();

        for az_idx in 0..output.azimuth_count() {
            for gate_idx in 0..output.gate_count() {
                let (val, status) = output.get(az_idx, gate_idx);

                if status == GateStatus::Valid {
                    let below_min = self.min.is_some_and(|m| val < m);
                    let above_max = self.max.is_some_and(|m| val > m);

                    if below_min || above_max {
                        output.set(az_idx, gate_idx, 0.0, GateStatus::NoData);
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
        let mut field =
            SweepField::new_empty("Test", "dBZ", 0.5, vec![0.0, 1.0, 2.0], 1.0, 2.0, 0.25, 5);

        // Set some test values
        for az in 0..3 {
            for gate in 0..5 {
                let value = (az * 5 + gate) as f32 * 5.0; // 0, 5, 10, ..., 70
                field.set(az, gate, value, GateStatus::Valid);
            }
        }

        field
    }

    #[test]
    fn test_threshold_min_only() {
        let field = make_test_field();
        let filter = ThresholdFilter {
            min: Some(20.0),
            max: None,
        };

        let result = filter.process(&field).unwrap();

        // Values below 20 should be masked
        for az in 0..3 {
            for gate in 0..5 {
                let original_value = (az * 5 + gate) as f32 * 5.0;
                let (val, status) = result.get(az, gate);
                if original_value < 20.0 {
                    assert_eq!(status, GateStatus::NoData);
                    assert_eq!(val, 0.0);
                } else {
                    assert_eq!(status, GateStatus::Valid);
                    assert_eq!(val, original_value);
                }
            }
        }
    }

    #[test]
    fn test_threshold_max_only() {
        let field = make_test_field();
        let filter = ThresholdFilter {
            min: None,
            max: Some(40.0),
        };

        let result = filter.process(&field).unwrap();

        for az in 0..3 {
            for gate in 0..5 {
                let original_value = (az * 5 + gate) as f32 * 5.0;
                let (_, status) = result.get(az, gate);
                if original_value > 40.0 {
                    assert_eq!(status, GateStatus::NoData);
                } else {
                    assert_eq!(status, GateStatus::Valid);
                }
            }
        }
    }

    #[test]
    fn test_threshold_both_bounds() {
        let field = make_test_field();
        let filter = ThresholdFilter {
            min: Some(15.0),
            max: Some(50.0),
        };

        let result = filter.process(&field).unwrap();

        for az in 0..3 {
            for gate in 0..5 {
                let original_value = (az * 5 + gate) as f32 * 5.0;
                let (_, status) = result.get(az, gate);
                if original_value < 15.0 || original_value > 50.0 {
                    assert_eq!(status, GateStatus::NoData);
                } else {
                    assert_eq!(status, GateStatus::Valid);
                }
            }
        }
    }

    #[test]
    fn test_threshold_preserves_nodata() {
        let mut field = make_test_field();
        // Mark some gates as already NoData
        field.set(1, 2, 0.0, GateStatus::NoData);

        let filter = ThresholdFilter {
            min: Some(0.0),
            max: None,
        };

        let result = filter.process(&field).unwrap();

        // The NoData gate should remain NoData
        let (_, status) = result.get(1, 2);
        assert_eq!(status, GateStatus::NoData);
    }

    #[test]
    fn test_threshold_no_bounds() {
        let field = make_test_field();
        let filter = ThresholdFilter {
            min: None,
            max: None,
        };

        let result = filter.process(&field).unwrap();

        // All valid values should remain valid
        for az in 0..3 {
            for gate in 0..5 {
                let (orig_val, _) = field.get(az, gate);
                let (result_val, result_status) = result.get(az, gate);
                assert_eq!(result_val, orig_val);
                assert_eq!(result_status, GateStatus::Valid);
            }
        }
    }
}
