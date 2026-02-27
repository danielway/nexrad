use crate::result::{Error, Result};
use crate::SweepProcessor;
use nexrad_model::data::{GateStatus, SweepField};

/// Correlation coefficient-based clutter removal.
///
/// Masks gates in a target field (e.g., reflectivity) where the corresponding
/// gate in a correlation coefficient (CC) field falls below a threshold. This is
/// a standard technique for removing non-meteorological echoes such as ground
/// clutter, biological targets, and debris.
///
/// Typical CC thresholds range from 0.85 to 0.95. Pure precipitation has
/// CC values near 1.0, while clutter and biological targets tend to have
/// CC well below 0.9.
///
/// # Example
///
/// ```ignore
/// use nexrad_process::filter::CorrelationCoefficientFilter;
///
/// let cc_field = SweepField::from_radials(radials, Product::CorrelationCoefficient).unwrap();
/// let filter = CorrelationCoefficientFilter::new(0.90, cc_field);
/// let cleaned = filter.process(&reflectivity_field)?;
/// ```
pub struct CorrelationCoefficientFilter {
    /// Gates with CC below this value are masked in the target field.
    threshold: f32,
    /// The correlation coefficient field to compare against.
    cc_field: SweepField,
}

impl CorrelationCoefficientFilter {
    /// Create a new CC filter with the given threshold and CC field.
    ///
    /// # Errors
    ///
    /// Returns an error if the threshold is not in (0, 1].
    pub fn new(threshold: f32, cc_field: SweepField) -> Result<Self> {
        if threshold <= 0.0 || threshold > 1.0 {
            return Err(Error::InvalidParameter(
                "CC threshold must be in (0, 1]".to_string(),
            ));
        }
        Ok(Self {
            threshold,
            cc_field,
        })
    }
}

impl SweepProcessor for CorrelationCoefficientFilter {
    fn name(&self) -> &str {
        "CorrelationCoefficientFilter"
    }

    fn process(&self, input: &SweepField) -> Result<SweepField> {
        if input.azimuth_count() != self.cc_field.azimuth_count() {
            return Err(Error::InvalidGeometry(format!(
                "CC field azimuth count ({}) does not match input field ({})",
                self.cc_field.azimuth_count(),
                input.azimuth_count(),
            )));
        }

        // When gate counts differ (e.g., REF at 460 km vs CC at 300 km), apply
        // the filter over the shared range and leave remaining gates unchanged.
        let shared_gates = input.gate_count().min(self.cc_field.gate_count());

        let mut output = input.clone();

        for az_idx in 0..input.azimuth_count() {
            for gate_idx in 0..shared_gates {
                let (_, input_status) = input.get(az_idx, gate_idx);
                if input_status != GateStatus::Valid {
                    continue;
                }

                let (cc_val, cc_status) = self.cc_field.get(az_idx, gate_idx);

                // If CC is invalid or below threshold, mask the target gate
                if cc_status != GateStatus::Valid || cc_val < self.threshold {
                    output.set(az_idx, gate_idx, 0.0, GateStatus::NoData);
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fields(gate_count: usize) -> (SweepField, SweepField) {
        let azimuths = vec![0.0, 1.0, 2.0];

        let mut target = SweepField::new_empty(
            "Reflectivity",
            "dBZ",
            0.5,
            azimuths.clone(),
            1.0,
            2.0,
            0.25,
            gate_count,
        );

        let mut cc = SweepField::new_empty("CC", "", 0.5, azimuths, 1.0, 2.0, 0.25, gate_count);

        // Fill target with valid data
        for az in 0..3 {
            for gate in 0..gate_count {
                target.set(az, gate, 30.0, GateStatus::Valid);
                cc.set(az, gate, 0.98, GateStatus::Valid);
            }
        }

        (target, cc)
    }

    #[test]
    fn test_cc_filter_preserves_high_cc() {
        let (target, cc) = make_fields(5);
        let filter = CorrelationCoefficientFilter::new(0.90, cc).unwrap();
        let result = filter.process(&target).unwrap();

        for az in 0..3 {
            for gate in 0..5 {
                let (val, status) = result.get(az, gate);
                assert_eq!(val, 30.0);
                assert_eq!(status, GateStatus::Valid);
            }
        }
    }

    #[test]
    fn test_cc_filter_masks_low_cc() {
        let (target, mut cc) = make_fields(5);
        // Set low CC for some gates
        cc.set(1, 2, 0.5, GateStatus::Valid);
        cc.set(1, 3, 0.8, GateStatus::Valid);

        let filter = CorrelationCoefficientFilter::new(0.90, cc).unwrap();
        let result = filter.process(&target).unwrap();

        // Low CC gates should be masked
        let (_, status) = result.get(1, 2);
        assert_eq!(status, GateStatus::NoData);

        let (_, status) = result.get(1, 3);
        assert_eq!(status, GateStatus::NoData);

        // High CC gate should be preserved
        let (val, status) = result.get(1, 1);
        assert_eq!(val, 30.0);
        assert_eq!(status, GateStatus::Valid);
    }

    #[test]
    fn test_cc_filter_masks_invalid_cc() {
        let (target, mut cc) = make_fields(5);
        cc.set(0, 0, 0.0, GateStatus::NoData);

        let filter = CorrelationCoefficientFilter::new(0.90, cc).unwrap();
        let result = filter.process(&target).unwrap();

        let (_, status) = result.get(0, 0);
        assert_eq!(status, GateStatus::NoData);
    }

    #[test]
    fn test_cc_filter_invalid_threshold() {
        let (_, cc) = make_fields(5);
        assert!(CorrelationCoefficientFilter::new(0.0, cc).is_err());

        let (_, cc) = make_fields(5);
        assert!(CorrelationCoefficientFilter::new(1.5, cc).is_err());
    }

    #[test]
    fn test_cc_filter_different_gate_counts_ok() {
        // REF with more gates than CC (common: REF 460 km vs CC 300 km)
        let (target, _) = make_fields(10);
        let (_, cc_shorter) = make_fields(5);

        let filter = CorrelationCoefficientFilter::new(0.90, cc_shorter).unwrap();
        let result = filter.process(&target).unwrap();

        // Gates in the shared range should be preserved (high CC)
        let (val, status) = result.get(0, 2);
        assert_eq!(val, 30.0);
        assert_eq!(status, GateStatus::Valid);

        // Gates beyond CC range should be left unchanged
        let (val, status) = result.get(0, 7);
        assert_eq!(val, 30.0);
        assert_eq!(status, GateStatus::Valid);
    }

    #[test]
    fn test_cc_filter_azimuth_mismatch_error() {
        // Different azimuth counts should still error
        let azimuths_3 = vec![0.0, 1.0, 2.0];
        let azimuths_4 = vec![0.0, 1.0, 2.0, 3.0];

        let mut target =
            SweepField::new_empty("Reflectivity", "dBZ", 0.5, azimuths_3, 1.0, 2.0, 0.25, 5);
        let mut cc = SweepField::new_empty("CC", "", 0.5, azimuths_4, 1.0, 2.0, 0.25, 5);

        for az in 0..3 {
            for gate in 0..5 {
                target.set(az, gate, 30.0, GateStatus::Valid);
            }
        }
        for az in 0..4 {
            for gate in 0..5 {
                cc.set(az, gate, 0.98, GateStatus::Valid);
            }
        }

        let filter = CorrelationCoefficientFilter::new(0.90, cc).unwrap();
        assert!(filter.process(&target).is_err());
    }
}
