//! Polar sweep type for radar data.

use crate::meta::Site;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A polar sweep of radar data with uniform geometry.
///
/// This is the canonical interchange type for polar radar data. It represents
/// a single sweep (360-degree rotation) with data values at each (ray, gate)
/// position in a uniform grid.
///
/// # Data Layout
///
/// Data is stored in row-major order where rays are rows and gates are columns:
/// `values[ray * gate_count + gate]`
///
/// This layout is cache-friendly for iterating along rays (radial traversal),
/// which matches how NEXRAD data arrives and how polar rendering typically works.
///
/// # Invalid Values
///
/// Invalid data (below threshold, range folded, etc.) is represented as `f32::NAN`
/// in the values array. Consumers should check for NaN when processing data.
///
/// # Example
///
/// ```
/// use nexrad_model::field::PolarSweep;
///
/// // Create a simple sweep with 360 rays and 100 gates
/// let azimuths: Vec<f32> = (0..360).map(|i| i as f32).collect();
/// let values = vec![0.0f32; 360 * 100];
///
/// let sweep = PolarSweep::new(
///     0.5,           // elevation angle in degrees
///     azimuths,
///     2125.0,        // first gate range in meters
///     250.0,         // gate size in meters
///     100,           // gate count
///     values,
/// );
///
/// assert_eq!(sweep.ray_count(), 360);
/// assert_eq!(sweep.gate_count(), 100);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PolarSweep<T> {
    elevation_deg: f32,
    azimuth_deg: Vec<f32>,
    first_gate_range_m: f32,
    gate_size_m: f32,
    gate_count: usize,
    values: Vec<T>,
    site: Option<Site>,
}

impl<T> PolarSweep<T> {
    /// Creates a new PolarSweep with the given geometry and data.
    ///
    /// # Arguments
    ///
    /// * `elevation_deg` - Elevation angle in degrees above horizontal
    /// * `azimuth_deg` - Azimuth angles in degrees clockwise from north, one per ray
    /// * `first_gate_range_m` - Distance to center of first gate in meters
    /// * `gate_size_m` - Distance between gate centers in meters
    /// * `gate_count` - Number of gates per ray
    /// * `values` - Data values in row-major order (ray, gate)
    ///
    /// # Panics
    ///
    /// Panics if `values.len() != azimuth_deg.len() * gate_count`.
    pub fn new(
        elevation_deg: f32,
        azimuth_deg: Vec<f32>,
        first_gate_range_m: f32,
        gate_size_m: f32,
        gate_count: usize,
        values: Vec<T>,
    ) -> Self {
        let ray_count = azimuth_deg.len();
        assert_eq!(
            values.len(),
            ray_count * gate_count,
            "values length {} does not match ray_count {} * gate_count {} = {}",
            values.len(),
            ray_count,
            gate_count,
            ray_count * gate_count
        );
        Self {
            elevation_deg,
            azimuth_deg,
            first_gate_range_m,
            gate_size_m,
            gate_count,
            values,
            site: None,
        }
    }

    /// Elevation angle in degrees above horizontal.
    pub fn elevation_deg(&self) -> f32 {
        self.elevation_deg
    }

    /// Azimuth angles in degrees clockwise from north, one per ray.
    pub fn azimuth_deg(&self) -> &[f32] {
        &self.azimuth_deg
    }

    /// Distance to center of first gate in meters.
    pub fn first_gate_range_m(&self) -> f32 {
        self.first_gate_range_m
    }

    /// Distance between gate centers in meters.
    pub fn gate_size_m(&self) -> f32 {
        self.gate_size_m
    }

    /// Number of rays (azimuth angles) in the sweep.
    pub fn ray_count(&self) -> usize {
        self.azimuth_deg.len()
    }

    /// Number of gates per ray.
    pub fn gate_count(&self) -> usize {
        self.gate_count
    }

    /// Returns a reference to the data values.
    pub fn values(&self) -> &[T] {
        &self.values
    }

    /// Returns a mutable reference to the data values.
    pub fn values_mut(&mut self) -> &mut [T] {
        &mut self.values
    }

    /// Consumes the sweep and returns the values vector.
    pub fn into_values(self) -> Vec<T> {
        self.values
    }

    /// Optional radar site metadata.
    pub fn site(&self) -> Option<&Site> {
        self.site.as_ref()
    }

    /// Sets the radar site metadata.
    pub fn with_site(mut self, site: Site) -> Self {
        self.site = Some(site);
        self
    }

    /// Returns the linear index for (ray, gate) coordinates.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if ray >= ray_count or gate >= gate_count.
    #[inline]
    pub fn idx(&self, ray: usize, gate: usize) -> usize {
        debug_assert!(
            ray < self.ray_count(),
            "ray={} >= ray_count={}",
            ray,
            self.ray_count()
        );
        debug_assert!(
            gate < self.gate_count,
            "gate={} >= gate_count={}",
            gate,
            self.gate_count
        );
        ray * self.gate_count + gate
    }

    /// Returns a reference to the value at (ray, gate).
    ///
    /// # Panics
    ///
    /// Panics if ray >= ray_count or gate >= gate_count.
    #[inline]
    pub fn get(&self, ray: usize, gate: usize) -> &T {
        &self.values[self.idx(ray, gate)]
    }

    /// Returns a mutable reference to the value at (ray, gate).
    ///
    /// # Panics
    ///
    /// Panics if ray >= ray_count or gate >= gate_count.
    #[inline]
    pub fn get_mut(&mut self, ray: usize, gate: usize) -> &mut T {
        let idx = self.idx(ray, gate);
        &mut self.values[idx]
    }

    /// Returns the range to a gate center in meters.
    pub fn gate_range_m(&self, gate: usize) -> f32 {
        self.first_gate_range_m + gate as f32 * self.gate_size_m
    }

    /// Returns the maximum range in meters (outer edge of last gate).
    pub fn max_range_m(&self) -> f32 {
        if self.gate_count == 0 {
            self.first_gate_range_m
        } else {
            self.first_gate_range_m
                + (self.gate_count - 1) as f32 * self.gate_size_m
                + self.gate_size_m / 2.0
        }
    }

    /// Total number of data points in the sweep.
    pub fn data_count(&self) -> usize {
        self.values.len()
    }
}

impl<T: Copy> PolarSweep<T> {
    /// Returns an iterator over rays, yielding (azimuth_deg, &[T]) for each ray.
    pub fn rays(&self) -> impl Iterator<Item = (f32, &[T])> {
        self.azimuth_deg.iter().enumerate().map(move |(i, &az)| {
            let start = i * self.gate_count;
            let end = start + self.gate_count;
            (az, &self.values[start..end])
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polar_sweep_creation() {
        let sweep = PolarSweep::new(
            0.5,
            vec![0.0, 1.0, 2.0],
            2125.0,
            250.0,
            100,
            vec![0.0f32; 300],
        );

        assert_eq!(sweep.ray_count(), 3);
        assert_eq!(sweep.gate_count(), 100);
        assert_eq!(sweep.elevation_deg(), 0.5);
        assert_eq!(sweep.first_gate_range_m(), 2125.0);
        assert_eq!(sweep.gate_size_m(), 250.0);
    }

    #[test]
    fn test_polar_sweep_indexing() {
        let sweep = PolarSweep::new(
            0.5,
            vec![0.0, 1.0],
            2125.0,
            250.0,
            3,
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        );

        assert_eq!(*sweep.get(0, 0), 1.0);
        assert_eq!(*sweep.get(0, 2), 3.0);
        assert_eq!(*sweep.get(1, 0), 4.0);
        assert_eq!(sweep.idx(1, 2), 5);
        assert_eq!(*sweep.get(1, 2), 6.0);
    }

    #[test]
    fn test_polar_sweep_range_calculations() {
        let sweep = PolarSweep::new(
            0.5,
            vec![0.0],
            2000.0, // first gate at 2km
            250.0,  // 250m gates
            100,
            vec![0.0f32; 100],
        );

        assert_eq!(sweep.gate_range_m(0), 2000.0);
        assert_eq!(sweep.gate_range_m(1), 2250.0);
        assert_eq!(sweep.gate_range_m(99), 2000.0 + 99.0 * 250.0);

        // Max range = last gate center + half gate size
        let expected_max = 2000.0 + 99.0 * 250.0 + 125.0;
        assert!((sweep.max_range_m() - expected_max).abs() < 0.01);
    }

    #[test]
    fn test_polar_sweep_with_site() {
        let site = Site::new(*b"KTLX", 35.3331, -97.2778, 370, 386);
        let sweep = PolarSweep::new(0.5, vec![0.0], 2125.0, 250.0, 10, vec![0.0f32; 10])
            .with_site(site.clone());

        assert!(sweep.site().is_some());
        assert_eq!(
            sweep.site().map(|s| s.identifier_string()),
            Some("KTLX".to_string())
        );
    }

    #[test]
    fn test_polar_sweep_rays_iterator() {
        let sweep = PolarSweep::new(
            0.5,
            vec![0.0, 90.0, 180.0],
            2125.0,
            250.0,
            2,
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        );

        let rays: Vec<_> = sweep.rays().collect();
        assert_eq!(rays.len(), 3);
        assert_eq!(rays[0], (0.0, &[1.0, 2.0][..]));
        assert_eq!(rays[1], (90.0, &[3.0, 4.0][..]));
        assert_eq!(rays[2], (180.0, &[5.0, 6.0][..]));
    }

    #[test]
    #[should_panic]
    fn test_polar_sweep_size_mismatch() {
        let _ = PolarSweep::new(0.5, vec![0.0, 1.0], 2125.0, 250.0, 100, vec![0.0f32; 50]);
    }
}
