use crate::result::{check_finite, check_product, Error, Result};
use crate::SweepProcessor;
use nexrad_model::data::{GateStatus, Product, SweepField};

/// Correlation coefficient-based clutter removal.
///
/// Masks gates in a target field (e.g., reflectivity) where the corresponding
/// gate in a correlation coefficient (CC) field is not valid or falls below a threshold.
/// Masked gates are marked [`GateStatus::Filtered`] and keep their values; gates that are
/// not [`GateStatus::Valid`] in the target field pass through unchanged. This is
/// a standard technique for removing non-meteorological echoes such as ground
/// clutter, biological targets, and debris.
///
/// Typical CC thresholds range from 0.85 to 0.95. Pure precipitation has
/// CC values near 1.0, while clutter and biological targets tend to have
/// CC well below 0.9.
///
/// The input field and the correlation coefficient field must share their azimuths, first gate
/// range and gate interval, so that gates are compared at the same location. Their gate counts
/// may differ (for example reflectivity to 460 km and correlation coefficient to 300 km); the
/// filter applies over the shared range and leaves gates beyond it unchanged.
///
/// # Example
///
/// ```
/// use nexrad_model::data::{Product, SweepField};
/// use nexrad_process::filter::CorrelationCoefficientFilter;
/// use nexrad_process::SweepProcessor;
/// # use nexrad_model::data::{MomentData, Radial, Sweep};
/// # let reflectivity = MomentData::from_fixed_point(
/// #     Product::Reflectivity, 3, 2000, 250, 8, 2.0, 66.0, vec![100, 110, 120],
/// # )?;
/// # let cc = MomentData::from_fixed_point(
/// #     Product::CorrelationCoefficient, 3, 2000, 250, 8, 200.0, 0.0, vec![196, 150, 198],
/// # )?;
/// # let radial = Radial::builder(0, 0.0, 1, 0.5).moment(reflectivity).moment(cc).build()?;
/// # let sweep = Sweep::new(1, vec![radial])?;
/// # let radials = sweep.radials();
/// # let reflectivity_field = SweepField::from_radials(radials, Product::Reflectivity).unwrap();
///
/// let cc_field = SweepField::from_radials(radials, Product::CorrelationCoefficient).unwrap();
/// let filter = CorrelationCoefficientFilter::new(0.90, cc_field)?;
/// let cleaned = filter.process(&reflectivity_field)?;
/// # Ok::<(), nexrad_process::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct CorrelationCoefficientFilter {
    /// Gates with CC below this value are masked in the target field.
    threshold: f32,
    /// The correlation coefficient field to compare against.
    correlation_coefficient: SweepField,
}

impl CorrelationCoefficientFilter {
    /// Create a new CC filter with the given threshold and correlation coefficient field.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if the threshold is not finite or not in
    /// `0.0..=1.0`, and [`Error::UnexpectedProduct`] if `correlation_coefficient` does not hold
    /// [`Product::CorrelationCoefficient`].
    pub fn new(threshold: f32, correlation_coefficient: SweepField) -> Result<Self> {
        check_finite("threshold", threshold)?;
        if !(0.0..=1.0).contains(&threshold) {
            return Err(Error::InvalidParameter {
                parameter: "threshold",
                reason: "must be in 0.0..=1.0",
            });
        }
        check_product(&correlation_coefficient, Product::CorrelationCoefficient)?;
        Ok(Self {
            threshold,
            correlation_coefficient,
        })
    }

    /// Gates with a correlation coefficient below this value are masked.
    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// The correlation coefficient field gates are compared against.
    pub fn correlation_coefficient(&self) -> &SweepField {
        &self.correlation_coefficient
    }
}

impl SweepProcessor for CorrelationCoefficientFilter {
    fn name(&self) -> &str {
        "CorrelationCoefficientFilter"
    }

    /// # Errors
    ///
    /// Returns [`Error::GeometryMismatch`] if the input field's azimuths, first gate range or
    /// gate interval differ from the correlation coefficient field's.
    fn process(&self, input: &SweepField) -> Result<SweepField> {
        let cc = &self.correlation_coefficient;
        let mismatch = |parameter| Err(Error::GeometryMismatch { parameter });
        if input.azimuth_angles_degrees() != cc.azimuth_angles_degrees() {
            return mismatch("azimuth_angles_degrees");
        }
        if input.first_gate_range_km() != cc.first_gate_range_km() {
            return mismatch("first_gate_range_km");
        }
        if input.gate_interval_km() != cc.gate_interval_km() {
            return mismatch("gate_interval_km");
        }

        // When gate counts differ (e.g., REF at 460 km vs CC at 300 km), apply
        // the filter over the shared range and leave remaining gates unchanged.
        let shared_gates = input.gate_count().min(cc.gate_count());

        let mut output = input.clone();

        for az_idx in 0..input.azimuth_count() {
            for gate_idx in 0..shared_gates {
                let Some((input_val, input_status)) = input.get(az_idx, gate_idx) else {
                    continue;
                };
                if input_status != GateStatus::Valid {
                    continue;
                }

                let Some((cc_val, cc_status)) = cc.get(az_idx, gate_idx) else {
                    continue;
                };

                // If CC is invalid or below threshold, mask the target gate
                if cc_status != GateStatus::Valid || cc_val < self.threshold {
                    output.set(az_idx, gate_idx, input_val, GateStatus::Filtered)?;
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

        let mut target = SweepField::builder("Reflectivity", "dBZ")
            .elevation_angle_degrees(0.5)
            .azimuth_angles_degrees(azimuths.clone())
            .first_gate_range_km(2.0)
            .gate_interval_km(0.25)
            .gate_count(gate_count)
            .build()
            .expect("valid geometry");

        let mut cc = SweepField::builder("CC", "")
            .product(Product::CorrelationCoefficient)
            .elevation_angle_degrees(0.5)
            .azimuth_angles_degrees(azimuths)
            .first_gate_range_km(2.0)
            .gate_interval_km(0.25)
            .gate_count(gate_count)
            .build()
            .expect("valid geometry");

        // Fill target with valid data
        for az in 0..3 {
            for gate in 0..gate_count {
                target.set(az, gate, 30.0, GateStatus::Valid).unwrap();
                cc.set(az, gate, 0.98, GateStatus::Valid).unwrap();
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
                let (val, status) = result.get(az, gate).unwrap();
                assert_eq!(val, 30.0);
                assert_eq!(status, GateStatus::Valid);
            }
        }
    }

    #[test]
    fn test_cc_filter_masks_low_cc() {
        let (target, mut cc) = make_fields(5);
        // Set low CC for some gates
        cc.set(1, 2, 0.5, GateStatus::Valid).unwrap();
        cc.set(1, 3, 0.8, GateStatus::Valid).unwrap();

        let filter = CorrelationCoefficientFilter::new(0.90, cc).unwrap();
        let result = filter.process(&target).unwrap();

        // Low CC gates are masked and keep their values
        assert_eq!(result.get(1, 2), Some((30.0, GateStatus::Filtered)));
        assert_eq!(result.get(1, 3), Some((30.0, GateStatus::Filtered)));

        // High CC gate should be preserved
        let (val, status) = result.get(1, 1).unwrap();
        assert_eq!(val, 30.0);
        assert_eq!(status, GateStatus::Valid);
    }

    #[test]
    fn test_cc_filter_masks_invalid_cc() {
        let (target, mut cc) = make_fields(5);
        cc.set(0, 0, 0.0, GateStatus::NoData).unwrap();

        let filter = CorrelationCoefficientFilter::new(0.90, cc).unwrap();
        let result = filter.process(&target).unwrap();

        assert_eq!(result.get(0, 0), Some((30.0, GateStatus::Filtered)));
    }

    #[test]
    fn test_cc_filter_invalid_threshold() {
        for threshold in [-0.1, 1.5, f32::NAN, f32::INFINITY] {
            let (_, cc) = make_fields(5);
            assert!(
                matches!(
                    CorrelationCoefficientFilter::new(threshold, cc),
                    Err(Error::InvalidParameter {
                        parameter: "threshold",
                        ..
                    })
                ),
                "threshold {threshold} should be rejected"
            );
        }

        let (_, cc) = make_fields(5);
        let filter = CorrelationCoefficientFilter::new(0.0, cc.clone()).unwrap();
        assert_eq!(filter.threshold(), 0.0);
        assert_eq!(filter.correlation_coefficient(), &cc);
        assert!(CorrelationCoefficientFilter::new(1.0, cc).is_ok());
    }

    #[test]
    fn test_cc_filter_requires_correlation_coefficient() {
        let (target, _) = make_fields(5);
        assert_eq!(
            CorrelationCoefficientFilter::new(0.9, target),
            Err(Error::UnexpectedProduct {
                expected: Product::CorrelationCoefficient,
                actual: None,
            })
        );
    }

    #[test]
    fn test_cc_filter_geometry_mismatch() {
        let (target, cc) = make_fields(5);
        let rebuild = |builder: nexrad_model::data::SweepFieldBuilder| {
            let filter = CorrelationCoefficientFilter::new(0.9, builder.build().unwrap()).unwrap();
            match filter.process(&target) {
                Err(Error::GeometryMismatch { parameter }) => parameter,
                other => panic!("expected GeometryMismatch, got {other:?}"),
            }
        };
        let base = || {
            SweepField::builder("CC", "")
                .product(Product::CorrelationCoefficient)
                .elevation_angle_degrees(0.5)
                .first_gate_range_km(2.0)
                .gate_interval_km(0.25)
                .gate_count(5)
        };

        // Same azimuth count, rotated by half a degree.
        assert_eq!(
            rebuild(base().azimuth_angles_degrees(vec![0.5, 1.5, 2.5])),
            "azimuth_angles_degrees"
        );
        assert_eq!(
            rebuild(
                base()
                    .azimuth_angles_degrees(vec![0.0, 1.0, 2.0])
                    .first_gate_range_km(2.125)
            ),
            "first_gate_range_km"
        );
        assert_eq!(
            rebuild(
                base()
                    .azimuth_angles_degrees(vec![0.0, 1.0, 2.0])
                    .gate_interval_km(0.5)
            ),
            "gate_interval_km"
        );

        // The same geometry passes.
        let filter = CorrelationCoefficientFilter::new(0.9, cc).unwrap();
        assert!(filter.process(&target).is_ok());
    }

    #[test]
    fn test_cc_filter_different_gate_counts_ok() {
        // REF with more gates than CC (common: REF 460 km vs CC 300 km)
        let (target, _) = make_fields(10);
        let (_, cc_shorter) = make_fields(5);

        let filter = CorrelationCoefficientFilter::new(0.90, cc_shorter).unwrap();
        let result = filter.process(&target).unwrap();

        // Gates in the shared range should be preserved (high CC)
        let (val, status) = result.get(0, 2).unwrap();
        assert_eq!(val, 30.0);
        assert_eq!(status, GateStatus::Valid);

        // Gates beyond CC range should be left unchanged
        let (val, status) = result.get(0, 7).unwrap();
        assert_eq!(val, 30.0);
        assert_eq!(status, GateStatus::Valid);
    }

    #[test]
    fn test_cc_filter_azimuth_mismatch_error() {
        // Different azimuth counts should still error
        let azimuths_3 = vec![0.0, 1.0, 2.0];
        let azimuths_4 = vec![0.0, 1.0, 2.0, 3.0];

        let mut target = SweepField::builder("Reflectivity", "dBZ")
            .elevation_angle_degrees(0.5)
            .azimuth_angles_degrees(azimuths_3)
            .first_gate_range_km(2.0)
            .gate_interval_km(0.25)
            .gate_count(5)
            .build()
            .expect("valid geometry");
        let mut cc = SweepField::builder("CC", "")
            .product(Product::CorrelationCoefficient)
            .elevation_angle_degrees(0.5)
            .azimuth_angles_degrees(azimuths_4)
            .first_gate_range_km(2.0)
            .gate_interval_km(0.25)
            .gate_count(5)
            .build()
            .expect("valid geometry");

        for az in 0..3 {
            for gate in 0..5 {
                target.set(az, gate, 30.0, GateStatus::Valid).unwrap();
            }
        }
        for az in 0..4 {
            for gate in 0..5 {
                cc.set(az, gate, 0.98, GateStatus::Valid).unwrap();
            }
        }

        let filter = CorrelationCoefficientFilter::new(0.90, cc).unwrap();
        assert_eq!(
            filter.process(&target),
            Err(Error::GeometryMismatch {
                parameter: "azimuth_angles_degrees"
            })
        );
    }
}
