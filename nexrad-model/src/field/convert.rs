//! Conversion functions from radials to polar sweeps.

use super::{FieldError, PolarSweep};
use crate::data::{MomentData, MomentValue, Radial};

/// Selector for which radar product to extract from radials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProductSelector {
    /// Reflectivity (dBZ).
    Reflectivity,
    /// Radial velocity (m/s).
    Velocity,
    /// Spectrum width (m/s).
    SpectrumWidth,
    /// Differential reflectivity (dB).
    DifferentialReflectivity,
    /// Differential phase (degrees).
    DifferentialPhase,
    /// Correlation coefficient (unitless, 0-1).
    CorrelationCoefficient,
    /// Specific differential phase (degrees/km).
    SpecificDifferentialPhase,
}

impl ProductSelector {
    /// Extracts the moment data for this product from a radial.
    fn get_moment<'a>(&self, radial: &'a Radial) -> Option<&'a MomentData> {
        match self {
            Self::Reflectivity => radial.reflectivity(),
            Self::Velocity => radial.velocity(),
            Self::SpectrumWidth => radial.spectrum_width(),
            Self::DifferentialReflectivity => radial.differential_reflectivity(),
            Self::DifferentialPhase => radial.differential_phase(),
            Self::CorrelationCoefficient => radial.correlation_coefficient(),
            Self::SpecificDifferentialPhase => radial.specific_differential_phase(),
        }
    }
}

/// Converts a slice of radials to a PolarSweep for a specific product.
///
/// This function validates that all radials have consistent geometry and
/// extracts the specified product data. Invalid values (BelowThreshold,
/// RangeFolded) are represented as `f32::NAN` in the output.
///
/// # Arguments
///
/// * `radials` - Slice of radials to convert (should be from a single sweep)
/// * `product` - Which radar product to extract
///
/// # Returns
///
/// A `PolarSweep<f32>` containing the extracted data, or an error if:
/// - No radials provided
/// - The requested product is not available in the radials
/// - Radials have inconsistent geometry (gate count, interval, first gate range)
///
/// # Example
///
/// ```ignore
/// use nexrad_model::field::{radials_to_polar_sweep, ProductSelector};
/// use nexrad_model::data::Radial;
///
/// let radials: &[Radial] = /* ... */;
/// let sweep = radials_to_polar_sweep(radials, ProductSelector::Reflectivity)?;
/// println!("Sweep has {} rays and {} gates", sweep.ray_count(), sweep.gate_count());
/// ```
pub fn radials_to_polar_sweep(
    radials: &[Radial],
    product: ProductSelector,
) -> Result<PolarSweep<f32>, FieldError> {
    if radials.is_empty() {
        return Err(FieldError::EmptyInput);
    }

    // Get first radial's moment data to establish geometry
    let first_radial = &radials[0];
    let first_moment = product
        .get_moment(first_radial)
        .ok_or(FieldError::ProductNotAvailable)?;

    let gate_count = first_moment.gate_count() as usize;
    let first_gate_range_m = first_moment.first_gate_range_km() as f32 * 1000.0;
    let gate_size_m = first_moment.gate_interval_km() as f32 * 1000.0;
    let elevation_deg = first_radial.elevation_angle_degrees();

    let ray_count = radials.len();
    let total_samples = ray_count * gate_count;

    let mut azimuth_deg = Vec::with_capacity(ray_count);
    let mut values = Vec::with_capacity(total_samples);

    // Tolerance for geometry comparisons (1 meter)
    const TOLERANCE_M: f32 = 1.0;

    for (ray_idx, radial) in radials.iter().enumerate() {
        let moment = product
            .get_moment(radial)
            .ok_or(FieldError::ProductNotAvailable)?;

        // Validate consistent gate count
        if moment.gate_count() as usize != gate_count {
            return Err(FieldError::InconsistentGateCount {
                expected: gate_count as u16,
                found: moment.gate_count(),
                ray_index: ray_idx,
            });
        }

        // Validate consistent first gate range
        let this_first_range = moment.first_gate_range_km() as f32 * 1000.0;
        if (this_first_range - first_gate_range_m).abs() > TOLERANCE_M {
            return Err(FieldError::InconsistentFirstGateRange {
                expected: first_gate_range_m,
                found: this_first_range,
                ray_index: ray_idx,
            });
        }

        // Validate consistent gate interval
        let this_gate_size = moment.gate_interval_km() as f32 * 1000.0;
        if (this_gate_size - gate_size_m).abs() > TOLERANCE_M {
            return Err(FieldError::InconsistentGateInterval {
                expected: gate_size_m,
                found: this_gate_size,
                ray_index: ray_idx,
            });
        }

        azimuth_deg.push(radial.azimuth_angle_degrees());

        // Convert moment values
        for moment_value in moment.values() {
            let value = match moment_value {
                MomentValue::Value(v) => v,
                MomentValue::BelowThreshold | MomentValue::RangeFolded => f32::NAN,
            };
            values.push(value);
        }
    }

    Ok(PolarSweep::new(
        elevation_deg,
        azimuth_deg,
        first_gate_range_m,
        gate_size_m,
        gate_count,
        values,
    ))
}

impl TryFrom<(&[Radial], ProductSelector)> for PolarSweep<f32> {
    type Error = FieldError;

    fn try_from((radials, product): (&[Radial], ProductSelector)) -> Result<Self, Self::Error> {
        radials_to_polar_sweep(radials, product)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radials_to_polar_sweep_empty() {
        let result = radials_to_polar_sweep(&[], ProductSelector::Reflectivity);
        assert!(matches!(result, Err(FieldError::EmptyInput)));
    }

    // Integration tests with real radial data would go in the tests/ directory
}
