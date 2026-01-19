//! Polar sweep sampler.

use super::Sampler;
use nexrad_model::field::PolarSweep;
use std::f32::consts::PI;

/// Sampler for polar sweep data.
///
/// Converts (x, y) world coordinates to polar coordinates and samples
/// the sweep data using nearest-neighbor lookup.
pub struct PolarSampler<'a> {
    sweep: &'a PolarSweep<f32>,
}

impl<'a> PolarSampler<'a> {
    /// Creates a new sampler for the given polar sweep.
    pub fn new(sweep: &'a PolarSweep<f32>) -> Self {
        Self { sweep }
    }

    /// Returns a reference to the underlying sweep.
    pub fn sweep(&self) -> &'a PolarSweep<f32> {
        self.sweep
    }

    /// Finds the ray index for a given azimuth angle in degrees.
    ///
    /// Uses binary search for efficiency when azimuths are sorted.
    fn find_ray(&self, azimuth_deg: f32) -> Option<usize> {
        let azimuths = self.sweep.azimuth_deg();
        if azimuths.is_empty() {
            return None;
        }

        // Normalize azimuth to [0, 360)
        let azimuth = ((azimuth_deg % 360.0) + 360.0) % 360.0;

        // Calculate approximate spacing for tolerance
        let avg_spacing = if azimuths.len() > 1 {
            360.0 / azimuths.len() as f32
        } else {
            1.0
        };

        // Find closest azimuth
        let mut best_idx = 0;
        let mut best_diff = f32::MAX;

        for (i, &az) in azimuths.iter().enumerate() {
            // Handle wrap-around at 0/360
            let diff = angular_difference(azimuth, az);
            if diff < best_diff {
                best_diff = diff;
                best_idx = i;
            }
        }

        // Only return if within half the azimuth spacing (with small tolerance)
        if best_diff <= avg_spacing / 2.0 + 0.5 {
            Some(best_idx)
        } else {
            None
        }
    }
}

/// Calculate the minimum angular difference between two angles in degrees.
fn angular_difference(a: f32, b: f32) -> f32 {
    let diff = (a - b).abs();
    diff.min(360.0 - diff)
}

impl<'a> Sampler for PolarSampler<'a> {
    fn sample(&self, x_m: f32, y_m: f32) -> Option<f32> {
        // Convert to polar coordinates
        let range_m = (x_m * x_m + y_m * y_m).sqrt();

        // Check range bounds
        let first_gate = self.sweep.first_gate_range_m();
        let gate_size = self.sweep.gate_size_m();
        let max_range = self.sweep.max_range_m();

        if range_m < first_gate - gate_size / 2.0 || range_m > max_range {
            return None;
        }

        // Convert Cartesian to azimuth
        // atan2 gives angle from positive X axis (East), counter-clockwise
        // We need angle from North (positive Y), clockwise
        let azimuth_rad = x_m.atan2(y_m);
        let mut azimuth_deg = azimuth_rad * 180.0 / PI;

        // Normalize to [0, 360)
        if azimuth_deg < 0.0 {
            azimuth_deg += 360.0;
        }

        // Find gate index
        let gate_f = (range_m - first_gate) / gate_size;
        let gate = gate_f.round() as isize;

        if gate < 0 || gate >= self.sweep.gate_count() as isize {
            return None;
        }
        let gate = gate as usize;

        // Find ray index
        let ray = self.find_ray(azimuth_deg)?;

        // Get value and check for NaN
        let value = *self.sweep.get(ray, gate);
        if value.is_nan() {
            return None;
        }

        Some(value)
    }

    fn extent_m(&self) -> f32 {
        self.sweep.max_range_m()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angular_difference() {
        assert!((angular_difference(0.0, 10.0) - 10.0).abs() < 0.01);
        assert!((angular_difference(350.0, 10.0) - 20.0).abs() < 0.01);
        assert!((angular_difference(10.0, 350.0) - 20.0).abs() < 0.01);
        assert!((angular_difference(180.0, 180.0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_polar_sampler_out_of_range() {
        let sweep = PolarSweep::new(
            0.5,
            (0..360).map(|i| i as f32).collect(),
            2000.0, // first gate at 2km
            250.0,  // 250m gates
            100,
            vec![30.0f32; 36000],
        );
        let sampler = PolarSampler::new(&sweep);

        // Center should be out of range (inside first gate)
        assert!(sampler.sample(0.0, 0.0).is_none());

        // Way outside should be None
        assert!(sampler.sample(100000.0, 0.0).is_none());
    }

    #[test]
    fn test_polar_sampler_valid_point() {
        let sweep = PolarSweep::new(
            0.5,
            (0..360).map(|i| i as f32).collect(),
            2000.0,
            250.0,
            100,
            vec![42.0f32; 36000],
        );
        let sampler = PolarSampler::new(&sweep);

        // Point at ~3km north (azimuth 0)
        let result = sampler.sample(0.0, 3000.0);
        assert!(result.is_some());
        assert!((result.unwrap() - 42.0).abs() < 0.01);
    }

    #[test]
    fn test_polar_sampler_nan_value() {
        let mut values = vec![42.0f32; 36000];
        // Set some values to NAN
        values[0] = f32::NAN;

        let sweep = PolarSweep::new(
            0.5,
            (0..360).map(|i| i as f32).collect(),
            2000.0,
            250.0,
            100,
            values,
        );
        let sampler = PolarSampler::new(&sweep);

        // Point at first gate, azimuth 0 should be NaN -> None
        let result = sampler.sample(0.0, 2000.0);
        assert!(result.is_none());
    }
}
